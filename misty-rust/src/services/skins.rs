//! Skin et capes d'un compte Microsoft (API api.minecraftservices.com).

use super::{account, http};
use image::{imageops, Rgba, RgbaImage};
use serde::{Deserialize, Serialize};

const API: &str = "https://api.minecraftservices.com/minecraft/profile";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cape {
    pub id: String,
    pub nom: String,
    pub url: String,
    pub active: bool,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Profil {
    pub skin_url: String,
    pub fin: bool,
    pub capes: Vec<Cape>,
}

#[derive(Deserialize)]
struct ProfilJson {
    #[serde(default)]
    skins: Vec<SkinJson>,
    #[serde(default)]
    capes: Vec<CapeJson>,
}

#[derive(Deserialize)]
struct SkinJson {
    #[serde(default)]
    state: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    variant: String,
}

#[derive(Deserialize)]
struct CapeJson {
    id: String,
    #[serde(default)]
    state: String,
    #[serde(default)]
    url: String,
    #[serde(default)]
    alias: String,
}

/// Jeton valide (renouvelé si besoin).
async fn jeton() -> anyhow::Result<String> {
    Ok(account::connecter().await?.jeton)
}

/// Mojang refuse les demandes (HTTP 429). `attente` : secondes demandées par Mojang (0 = non précisé).
#[derive(Debug)]
pub struct Limite {
    pub attente: u64,
}

impl std::fmt::Display for Limite {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "HTTP 429 (trop de requêtes)")
    }
}

impl std::error::Error for Limite {}

/// Skin et capes actuels du compte.
pub async fn profil() -> anyhow::Result<Profil> {
    // Outil de développement : MISTY_FAKE_429=1 simule la limite de requêtes de Mojang (version debug seulement)
    if cfg!(debug_assertions) && std::env::var_os("MISTY_FAKE_429").is_some() {
        return Err(Limite { attente: 4 }.into());
    }
    let r = http::client().get(API).bearer_auth(jeton().await?).send().await?;
    if r.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        let attente = r.headers().get("retry-after").and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
        return Err(Limite { attente }.into());
    }
    let p: ProfilJson = r.error_for_status()?.json().await?;
    let skin = p.skins.iter().find(|s| s.state == "ACTIVE").or(p.skins.first());
    Ok(Profil {
        skin_url: skin.map(|s| s.url.clone()).unwrap_or_default(),
        fin: skin.map(|s| s.variant.eq_ignore_ascii_case("slim")).unwrap_or(false),
        capes: p
            .capes
            .into_iter()
            .map(|c| Cape { active: c.state == "ACTIVE", nom: majuscule(&c.alias), id: c.id, url: c.url })
            .collect(),
    })
}

fn majuscule(alias: &str) -> String {
    let mut c = alias.chars();
    match c.next() {
        Some(p) => p.to_uppercase().collect::<String>() + c.as_str(),
        None => "Cape".into(),
    }
}

/// Dossier où sont gardés les skins et capes déjà téléchargés.
fn dossier_cache() -> std::path::PathBuf {
    super::paths::data_dir().join("cache").join("skins")
}

/// Télécharge une texture Mojang, ou la relit sur le disque si on l'a déjà eue.
/// L'adresse d'une texture contient son empreinte : le contenu d'une même adresse ne change jamais.
pub async fn telecharger(url: &str) -> anyhow::Result<Vec<u8>> {
    let fichier = dossier_cache().join(format!("{:x}.png", md5::compute(url)));
    if let Ok(octets) = std::fs::read(&fichier) {
        if !octets.is_empty() {
            return Ok(octets);
        }
    }
    if url.starts_with("local:") {
        anyhow::bail!("skin local introuvable"); // pas une vraie adresse : seulement sur le disque
    }
    let octets = http::client().get(url).send().await?.error_for_status()?.bytes().await?.to_vec();
    let _ = std::fs::create_dir_all(dossier_cache());
    let _ = std::fs::write(&fichier, &octets);
    Ok(octets)
}

/// Dernier profil connu d'un compte (relu quand Mojang ne répond pas).
pub fn profil_en_cache(uuid: &str) -> Option<Profil> {
    if uuid.is_empty() {
        return None;
    }
    serde_json::from_str(&std::fs::read_to_string(dossier_cache().join(format!("profil-{uuid}.json"))).ok()?).ok()
}

pub fn enregistrer_profil(uuid: &str, profil: &Profil) {
    if uuid.is_empty() {
        return;
    }
    let _ = std::fs::create_dir_all(dossier_cache());
    let _ = std::fs::write(dossier_cache().join(format!("profil-{uuid}.json")), serde_json::to_string(profil).unwrap_or_default());
}

/// Après l'envoi d'un skin : on le garde sur le disque, pour l'afficher même si Mojang ne répond pas ensuite.
pub fn memoriser_skin_envoye(uuid: &str, png: &[u8], fin: bool) {
    let adresse = format!("local:{uuid}");
    let _ = std::fs::create_dir_all(dossier_cache());
    let _ = std::fs::write(dossier_cache().join(format!("{:x}.png", md5::compute(&adresse))), png);
    let mut profil = profil_en_cache(uuid).unwrap_or_default();
    profil.skin_url = adresse;
    profil.fin = fin;
    enregistrer_profil(uuid, &profil);
}

/// Steve en entier : toujours le même, calculé une seule fois puis gardé en mémoire.
pub fn steve_tour() -> Option<&'static Vec<RgbaImage>> {
    static STEVE: std::sync::OnceLock<Option<Vec<RgbaImage>>> = std::sync::OnceLock::new();
    STEVE.get_or_init(|| rendre_tour(include_bytes!("../../assets/steve.png"), false, None)).as_ref()
}

/// Envoie un nouveau skin (PNG 64×64 ou 64×32).
pub async fn changer_skin(png: Vec<u8>, fin: bool) -> anyhow::Result<()> {
    let limite = "----MistyLimite7MA4YWxkTrZu0gW";
    let mut corps: Vec<u8> = Vec::new();
    corps.extend_from_slice(
        format!(
            "--{limite}\r\nContent-Disposition: form-data; name=\"variant\"\r\n\r\n{}\r\n--{limite}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"skin.png\"\r\nContent-Type: image/png\r\n\r\n",
            if fin { "slim" } else { "classic" }
        )
        .as_bytes(),
    );
    corps.extend_from_slice(&png);
    corps.extend_from_slice(format!("\r\n--{limite}--\r\n").as_bytes());
    let r = http::client()
        .post(format!("{API}/skins"))
        .bearer_auth(jeton().await?)
        .header("Content-Type", format!("multipart/form-data; boundary={limite}"))
        .body(corps)
        .send()
        .await?;
    verifier(r).await
}

/// Active une cape (`None` = ne plus en porter).
pub async fn activer_cape(id: Option<&str>) -> anyhow::Result<()> {
    if cfg!(debug_assertions) && std::env::var_os("MISTY_FAKE_429").is_some() {
        return Err(Limite { attente: 2 }.into()); // simulation (voir profil)
    }
    let jeton = jeton().await?;
    let url = format!("{API}/capes/active");
    let r = match id {
        Some(id) => http::client().put(url).bearer_auth(jeton).json(&serde_json::json!({ "capeId": id })).send().await?,
        None => http::client().delete(url).bearer_auth(jeton).send().await?,
    };
    verifier(r).await
}

async fn verifier(r: reqwest::Response) -> anyhow::Result<()> {
    if r.status().is_success() {
        return Ok(());
    }
    if r.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
        let attente = r.headers().get("retry-after").and_then(|v| v.to_str().ok()).and_then(|v| v.parse::<u64>().ok()).unwrap_or(0);
        return Err(Limite { attente }.into());
    }
    let statut = r.status();
    let texte = r.text().await.unwrap_or_default();
    let detail = serde_json::from_str::<serde_json::Value>(&texte)
        .ok()
        .and_then(|v| v["errorMessage"].as_str().or(v["error"].as_str()).map(str::to_string))
        .unwrap_or(texte);
    anyhow::bail!("{statut} {detail}")
}

/// Un skin est valide s'il fait 64×64 ou 64×32 pixels.
pub fn skin_valide(png: &[u8]) -> bool {
    image::load_from_memory(png).map(|i| i.width() == 64 && (i.height() == 64 || i.height() == 32)).unwrap_or(false)
}

fn coller(dest: &mut RgbaImage, skin: &RgbaImage, src: (u32, u32, u32, u32), vers: (u32, u32), miroir: bool) {
    let (sx, sy, w, h) = src;
    for y in 0..h {
        for x in 0..w {
            let px = skin.get_pixel(if miroir { sx + w - 1 - x } else { sx + x }, sy + y);
            if px[3] == 0 {
                continue;
            }
            let (dx, dy) = (vers.0 + x, vers.1 + y);
            if px[3] == 255 {
                dest.put_pixel(dx, dy, *px);
            } else {
                let mut d = *dest.get_pixel(dx, dy);
                let a = px[3] as f32 / 255.0;
                for i in 0..3 {
                    d[i] = (px[i] as f32 * a + d[i] as f32 * (1.0 - a)) as u8;
                }
                d[3] = d[3].max(px[3]);
                dest.put_pixel(dx, dy, d);
            }
        }
    }
}

// ───────────────────────── 3D preview ─────────────────────────
// Tiny software ray caster: the player model is a handful of textured boxes,
// seen from the side with an orthographic camera that turns around the character.

/// Number of pre-rendered views for one full turn (keep in sync with ui/skin.slint).
pub const IMAGES_TOUR: usize = 36;
const ECHELLE: f32 = 9.0;
const LARGEUR: u32 = 224;
const HAUTEUR: u32 = 320;

/// Region of the texture used by one face of a box (x, y, width, height).
#[derive(Clone, Copy)]
struct Region(u32, u32, u32, u32);

struct Boite {
    min: [f32; 3],
    max: [f32; 3],
    /// +z, -z, -x, +x (front, back, character right, character left)
    faces: [Region; 4],
    /// the picture of each face is flipped horizontally
    miroir: bool,
    /// tilt around the X axis at the top of the box (radians): the bottom swings backwards
    incline: f32,
    /// second tilt, around the Z axis (radians): the bottom swings outward (elytra wings)
    roulis: f32,
    /// pivot of the tilts: (x, y, z)
    pivot: [f32; 3],
    /// top and bottom faces (only boxes that can show them when tilted, like the limbs)
    dessus: Option<[Region; 2]>,
}

/// Box of the player model, with the layout of the Minecraft skin (origin u,v and size w,h,d).
fn boite_skin(origine: (u32, u32), taille: (u32, u32, u32), min: [f32; 3], gonfle: f32, miroir: bool) -> Boite {
    let (u, v) = origine;
    let (w, h, d) = taille;
    let droite = Region(u, v + d, d, h);
    let devant = Region(u + d, v + d, w, h);
    let gauche = Region(u + d + w, v + d, d, h);
    let dos = Region(u + 2 * d + w, v + d, w, h);
    Boite {
        min: [min[0] - gonfle, min[1] - gonfle, min[2] - gonfle],
        max: [min[0] + w as f32 + gonfle, min[1] + h as f32 + gonfle, min[2] + d as f32 + gonfle],
        faces: if miroir { [devant, dos, gauche, droite] } else { [devant, dos, droite, gauche] },
        miroir,
        incline: 0.0,
        roulis: 0.0,
        pivot: [0.0, 0.0, 0.0],
        dessus: Some([Region(u + d, v, w, d), Region(u + d + w, v, w, d)]),
    }
}

/// Cape: a thin box behind the body, its outside faces backwards.
fn boite_cape(f: u32) -> Boite {
    Boite {
        min: [-5.0, 8.0, -3.25],
        max: [5.0, 24.0, -2.25],
        faces: [Region(12 * f, f, 10 * f, 16 * f), Region(f, f, 10 * f, 16 * f), Region(0, f, f, 16 * f), Region(11 * f, f, f, 16 * f)],
        miroir: false,
        incline: CAPE_INCLINAISON,
        roulis: 0.0,
        pivot: [0.0, 24.0, -2.25],
        dessus: None,
    }
}
/// Like on NameMC, the cape hangs diagonally instead of sticking to the body.
const CAPE_INCLINAISON: f32 = 0.25;

/// Entry point of a ray into a box: distance, axis and whether the face is on the positive side.
fn entrer(o: [f32; 3], d: [f32; 3], b: &Boite) -> Option<(f32, usize, bool)> {
    let (mut tmin, mut tmax) = (f32::NEG_INFINITY, f32::INFINITY);
    let (mut axe, mut positif) = (1, false);
    for i in 0..3 {
        if d[i].abs() < 1e-6 {
            if o[i] < b.min[i] || o[i] > b.max[i] {
                return None;
            }
            continue;
        }
        let t1 = (b.min[i] - o[i]) / d[i];
        let t2 = (b.max[i] - o[i]) / d[i];
        let (proche, loin) = if t1 < t2 { (t1, t2) } else { (t2, t1) };
        if proche > tmin {
            tmin = proche;
            axe = i;
            positif = d[i] < 0.0;
        }
        tmax = tmax.min(loin);
    }
    if tmin > tmax || tmax < 0.0 || (axe == 1 && b.dessus.is_none()) {
        return None;
    }
    Some((tmin, axe, positif))
}

fn echantillon(b: &Boite, texture: &RgbaImage, p: [f32; 3], axe: usize, positif: bool) -> Rgba<u8> {
    if axe == 1 {
        if let Some([haut, bas]) = b.dessus {
            let mut fu = (p[0] - b.min[0]) / (b.max[0] - b.min[0]);
            if b.miroir {
                fu = 1.0 - fu;
            }
            let profondeur = b.max[2] - b.min[2];
            let (Region(x, y, w, h), fv) = if positif { (haut, (p[2] - b.min[2]) / profondeur) } else { (bas, (b.max[2] - p[2]) / profondeur) };
            let tx = x + (fu.clamp(0.0, 0.9999) * w as f32) as u32;
            let ty = y + (fv.clamp(0.0, 0.9999) * h as f32) as u32;
            return *texture.get_pixel(tx.min(texture.width() - 1), ty.min(texture.height() - 1));
        }
    }
    let largeur = b.max[axe_largeur(axe)] - b.min[axe_largeur(axe)];
    let (face, mut fu) = match (axe, positif) {
        (2, true) => (0, (p[0] - b.min[0]) / largeur),
        (2, false) => (1, (b.max[0] - p[0]) / largeur),
        (0, false) => (2, (p[2] - b.min[2]) / largeur),
        _ => (3, (b.max[2] - p[2]) / largeur),
    };
    if b.miroir {
        fu = 1.0 - fu;
    }
    let fv = (b.max[1] - p[1]) / (b.max[1] - b.min[1]);
    let Region(x, y, w, h) = b.faces[face];
    let tx = x + ((fu.clamp(0.0, 0.9999)) * w as f32) as u32;
    let ty = y + ((fv.clamp(0.0, 0.9999)) * h as f32) as u32;
    *texture.get_pixel(tx.min(texture.width() - 1), ty.min(texture.height() - 1))
}

fn axe_largeur(axe: usize) -> usize {
    // faces facing z are as wide as the box is along x, faces facing x as the box is deep along z
    if axe == 2 {
        0
    } else {
        2
    }
}

/// The character turning on itself: `IMAGES_TOUR` views, the first one seen from the front.
/// The cape (texture bytes of the whole cape) is only visible from behind.
pub fn rendre_tour(png: &[u8], fin: bool, cape: Option<&[u8]>) -> Option<Vec<RgbaImage>> {
    let toutes: Vec<usize> = (0..IMAGES_TOUR).collect();
    rendre_vues(png, fin, cape, &toutes)
}

/// Seulement certaines vues du tour (ex. [0] : la face, pour afficher le personnage tout de suite).
pub fn rendre_vues(png: &[u8], fin: bool, cape: Option<&[u8]>, indices: &[usize]) -> Option<Vec<RgbaImage>> {
    rendre_vues_pose(png, fin, cape, indices, 0.0, false)
}

/// Le tour complet, avec des élytres à la place de la cape.
pub fn rendre_tour_elytres(png: &[u8], fin: bool, cape: Option<&[u8]>, elytres: bool) -> Option<Vec<RgbaImage>> {
    let toutes: Vec<usize> = (0..IMAGES_TOUR).collect();
    rendre_vues_pose(png, fin, cape, &toutes, 0.0, elytres)
}

/// Texture d'élytres par défaut, quand le joueur n'a pas de cape : la vraie texture du jeu,
/// lue dans une version de Minecraft déjà installée (puis gardée en cache), sinon une texture grise.
fn elytres_par_defaut() -> RgbaImage {
    static ELYTRES: std::sync::OnceLock<RgbaImage> = std::sync::OnceLock::new();
    ELYTRES.get_or_init(|| elytres_du_jeu().unwrap_or_else(elytres_grises)).clone()
}

fn elytres_du_jeu() -> Option<RgbaImage> {
    let cache = super::paths::data_dir().join("cache").join("elytra.png");
    if let Some(img) = std::fs::read(&cache).ok().and_then(|b| image::load_from_memory(&b).ok()) {
        return Some(img.to_rgba8());
    }
    const CHEMINS: [&str; 2] = ["assets/minecraft/textures/entity/equipment/wings/elytra.png", "assets/minecraft/textures/entity/elytra.png"];
    let versions = super::paths::data_dir().join("versions");
    for entree in std::fs::read_dir(versions).ok()?.flatten() {
        let nom = entree.file_name().to_string_lossy().to_string();
        let Ok(fichier) = std::fs::File::open(entree.path().join(format!("{nom}.jar"))) else { continue };
        let Ok(mut zip) = zip::ZipArchive::new(fichier) else { continue };
        for chemin in CHEMINS {
            let Ok(mut f) = zip.by_name(chemin) else { continue };
            let mut octets = Vec::new();
            if std::io::Read::read_to_end(&mut f, &mut octets).is_err() {
                continue;
            }
            if let Ok(img) = image::load_from_memory(&octets) {
                let img = img.to_rgba8();
                if img.width() == 64 && img.height() == 32 {
                    let _ = std::fs::create_dir_all(cache.parent()?);
                    let _ = std::fs::write(&cache, &octets);
                    return Some(img);
                }
            }
        }
    }
    None
}

fn elytres_grises() -> RgbaImage {
    RgbaImage::from_fn(64, 32, |x, y| {
        let n = ((x * 7 + y * 13 + (x * y) % 5 * 3) % 19) as u8;
        Rgba([104 + n, 106 + n, 120 + n, 255])
    })
}

/// Poses de la marche : nombre d'images du cycle, et nombre d'angles de vue (un tous les 30°).
pub const POSES_MARCHE: usize = 12;
pub const ANGLES_MARCHE: usize = 12;

/// Le personnage qui marche : POSES_MARCHE poses x ANGLES_MARCHE angles (pose d'abord).
pub fn rendre_marche(png: &[u8], fin: bool, cape: Option<&[u8]>, elytres: bool) -> Option<Vec<RgbaImage>> {
    let angles: Vec<usize> = (0..ANGLES_MARCHE).map(|a| a * IMAGES_TOUR / ANGLES_MARCHE).collect();
    let mut toutes = Vec::new();
    for pose in 0..POSES_MARCHE {
        let balancement = 0.5 * (pose as f32 * std::f32::consts::TAU / POSES_MARCHE as f32).sin();
        toutes.extend(rendre_vues_pose(png, fin, cape, &angles, balancement, elytres)?);
    }
    Some(toutes)
}

/// Comme `rendre_vues`, avec les bras et les jambes balancés de `balancement` radians (marche).
pub fn rendre_vues_pose(png: &[u8], fin: bool, cape: Option<&[u8]>, indices: &[usize], balancement: f32, elytres: bool) -> Option<Vec<RgbaImage>> {
    let skin = image::load_from_memory(png).ok()?.to_rgba8();
    if skin.width() != 64 || skin.height() < 32 {
        return None;
    }
    let mut cape_img = cape.and_then(|c| image::load_from_memory(c).ok()).map(|i| i.to_rgba8());
    if elytres {
        // les élytres utilisent la texture de la cape (64x32), ou une texture grise par défaut
        cape_img = Some(match cape_img {
            Some(c) if c.width() != 64 || c.height() != 32 => imageops::resize(&c, 64, 32, imageops::FilterType::Nearest),
            Some(c) => c,
            None => elytres_par_defaut(),
        });
    }
    let moderne = skin.height() >= 64;
    let bras: u32 = if fin { 3 } else { 4 };
    let b = bras as f32;

    let mut boites: Vec<Boite> = vec![
        boite_skin((0, 0), (8, 8, 8), [-4.0, 24.0, -4.0], 0.0, false),
        boite_skin((16, 16), (8, 12, 4), [-4.0, 12.0, -2.0], 0.0, false),
        boite_skin((40, 16), (bras, 12, 4), [-4.0 - b, 12.0, -2.0], 0.0, false),
        boite_skin((0, 16), (4, 12, 4), [-4.0, 0.0, -2.0], 0.0, false),
    ];
    if moderne {
        boites.push(boite_skin((32, 48), (bras, 12, 4), [4.0, 12.0, -2.0], 0.0, false));
        boites.push(boite_skin((16, 48), (4, 12, 4), [0.0, 0.0, -2.0], 0.0, false));
    } else {
        boites.push(boite_skin((40, 16), (bras, 12, 4), [4.0, 12.0, -2.0], 0.0, true));
        boites.push(boite_skin((0, 16), (4, 12, 4), [0.0, 0.0, -2.0], 0.0, true));
    }
    // Clothes layer, a little bigger than the base layer
    boites.push(boite_skin((32, 0), (8, 8, 8), [-4.0, 24.0, -4.0], 0.5, false));
    if moderne {
        boites.push(boite_skin((16, 32), (8, 12, 4), [-4.0, 12.0, -2.0], 0.25, false));
        boites.push(boite_skin((40, 32), (bras, 12, 4), [-4.0 - b, 12.0, -2.0], 0.25, false));
        boites.push(boite_skin((48, 48), (bras, 12, 4), [4.0, 12.0, -2.0], 0.25, false));
        boites.push(boite_skin((0, 32), (4, 12, 4), [-4.0, 0.0, -2.0], 0.25, false));
        boites.push(boite_skin((0, 48), (4, 12, 4), [0.0, 0.0, -2.0], 0.25, false));
    }
    let nb_skin = boites.len();
    if balancement != 0.0 {
        // bras droit et jambe gauche d'un côté, bras gauche et jambe droite de l'autre
        let mut pencher = |i: usize, angle: f32, y: f32| {
            if let Some(b) = boites.get_mut(i) {
                b.incline = angle;
                b.pivot = [0.0, y, 0.0];
            }
        };
        for (i, angle, y) in [(2, balancement, 22.0), (3, -balancement, 12.0), (4, -balancement, 22.0), (5, balancement, 12.0)] {
            pencher(i, angle, y);
        }
        if moderne {
            for (i, angle, y) in [(8, balancement, 22.0), (9, -balancement, 22.0), (10, -balancement, 12.0), (11, balancement, 12.0)] {
                pencher(i, angle, y);
            }
        }
    }
    let facteur = cape_img.as_ref().map(|c| (c.width() / 64).max(1)).unwrap_or(1);
    if elytres {
        // like in the game: the wings hang from the shoulders, tilted backwards and spread outward
        for (cote, miroir) in [(1.0f32, false), (-1.0, true)] {
            let mut aile = boite_skin((22, 0), (10, 20, 2), [-5.0, 4.0, -4.0], 0.0, miroir);
            aile.incline = 0.26;
            aile.roulis = 0.38 * cote;
            aile.pivot = [5.0 * cote, 24.0, -2.0];
            boites.push(aile);
        }
    } else if cape_img.is_some() {
        boites.push(boite_cape(facteur));
    }

    let rendre = |k: usize| -> RgbaImage {
        let a = k as f32 * std::f32::consts::TAU / IMAGES_TOUR as f32;
        let (s, c) = a.sin_cos();
        // the viewer is at (sin, 0, cos) and looks at the character; screen right is (cos, 0, -sin)
        let regard = [-s, 0.0, -c];
        let droite = [c, 0.0, -s];
        let mut vue = RgbaImage::from_pixel(LARGEUR, HAUTEUR, Rgba([0, 0, 0, 0]));
        for py in 0..HAUTEUR {
            let sy = (HAUTEUR as f32 / 2.0 - (py as f32 + 0.5)) / ECHELLE + 16.0;
            for px in 0..LARGEUR {
                let sx = (px as f32 + 0.5 - LARGEUR as f32 / 2.0) / ECHELLE;
                let o = [droite[0] * sx - regard[0] * 100.0, sy, droite[2] * sx - regard[2] * 100.0];
                let mut meilleur: Option<(f32, Rgba<u8>)> = None;
                for (i, boite) in boites.iter().enumerate() {
                    let (ol, dl) = if boite.incline == 0.0 && boite.roulis == 0.0 {
                        (o, regard)
                    } else {
                        // the ray in the frame of the tilted box (inverse rotations around the pivot)
                        let (sx, cx) = boite.incline.sin_cos();
                        let (sz, cz) = boite.roulis.sin_cos();
                        let inverse = |v: [f32; 3]| -> [f32; 3] {
                            let (x, y, z) = (v[0] * cz + v[1] * sz, -v[0] * sz + v[1] * cz, v[2]);
                            [x, y * cx + z * sx, -y * sx + z * cx]
                        };
                        let [px, py, pz] = boite.pivot;
                        let po = inverse([o[0] - px, o[1] - py, o[2] - pz]);
                        ([po[0] + px, po[1] + py, po[2] + pz], inverse(regard))
                    };
                    let Some((t, axe, positif)) = entrer(ol, dl, boite) else { continue };
                    if meilleur.map(|(m, _)| t >= m).unwrap_or(false) {
                        continue;
                    }
                    let p = [ol[0] + dl[0] * t, ol[1] + dl[1] * t, ol[2] + dl[2] * t];
                    let texture = if i >= nb_skin { cape_img.as_ref().unwrap() } else { &skin };
                    let couleur = echantillon(boite, texture, p, axe, positif);
                    if couleur[3] >= 128 {
                        meilleur = Some((t, Rgba([couleur[0], couleur[1], couleur[2], 255])));
                    }
                }
                if let Some((_, couleur)) = meilleur {
                    vue.put_pixel(px, py, couleur);
                }
            }
        }
        vue
    };

    // Les vues sont indépendantes : on les calcule en parallèle sur tous les cœurs
    let fils = std::thread::available_parallelism().map(|n| n.get()).unwrap_or(4).clamp(1, indices.len().max(1));
    let mut resultats: Vec<(usize, RgbaImage)> = Vec::with_capacity(indices.len());
    std::thread::scope(|sc| {
        let rendre = &rendre;
        let poignees: Vec<_> = (0..fils)
            .map(|t| sc.spawn(move || indices.iter().skip(t).step_by(fils).map(|&k| (k, rendre(k))).collect::<Vec<_>>()))
            .collect();
        for p in poignees {
            if let Ok(v) = p.join() {
                resultats.extend(v);
            }
        }
    });
    resultats.sort_by_key(|(k, _)| indices.iter().position(|i| i == k));
    Some(resultats.into_iter().map(|(_, v)| v).collect())
}
/// Tête du skin avec le chapeau, agrandie à 64×64 sans lissage.
pub fn rendre_tete(png: &[u8]) -> Option<RgbaImage> {
    let skin = image::load_from_memory(png).ok()?.to_rgba8();
    if skin.width() != 64 || skin.height() < 32 {
        return None;
    }
    let mut r = RgbaImage::from_pixel(8, 8, Rgba([0, 0, 0, 0]));
    coller(&mut r, &skin, (8, 8, 8, 8), (0, 0), false);
    coller(&mut r, &skin, (40, 8, 8, 8), (0, 0), false);
    Some(imageops::resize(&r, 64, 64, imageops::FilterType::Nearest))
}

/// Face extérieure d'une cape (10×16 sur une texture 64×32, ou un multiple).
pub fn rendre_cape(png: &[u8]) -> Option<RgbaImage> {
    let cape = image::load_from_memory(png).ok()?.to_rgba8();
    let f = (cape.width() / 64).max(1);
    if cape.width() < 22 * f || cape.height() < 17 * f {
        return None;
    }
    Some(imageops::crop_imm(&cape, f, f, 10 * f, 16 * f).to_image())
}
