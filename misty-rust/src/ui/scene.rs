//! Paysage du site misty-launcher.online (montagnes, sapins, lune, étoiles, nappes de brume),
//! repris en fond du launcher.
//!
//! Même générateur que le site (même graine = même paysage), mais au lieu d'un SVG on produit des
//! masques blancs : Slint les agrandit et les colore selon le thème (nuit, brume, Halloween, Noël).
//! Les animations (lune qui flotte, étoiles qui scintillent, brume qui dérive) sont faites côté Slint.
//!
//! Flou du fond (réglable dans les options) : Slint ne sait pas flouter un élément animé, alors on
//! floute chaque calque ici. Les calques restent séparés, donc les animations continuent.

use slint::{Image, Rgba8Pixel, SharedPixelBuffer};

/// Taille de la scène, en pixels du site (le launcher la met à l'échelle).
const W: i32 = 1600;
/// Taille d'un « pixel » de la scène.
const P: i32 = 8;
const COLS: i32 = W / P;
const ROWS: i32 = 55;

/// Position de la lune (fraction de la largeur) : entre le titre et la carte de version.
const LUNE_X: f32 = 0.56;
const LUNE_R: i32 = 44;

/// Image d'un calque, préparée hors du thread de l'interface (convertie en `Image` ensuite).
type Tampon = SharedPixelBuffer<Rgba8Pixel>;

/// Nappe de brume : image et position (pixels de la scène).
pub struct Nappe {
    pub image: Tampon,
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub couche: i32,
    pub periode_ms: i64,
    pub phase: f32,
    pub opacite: f32,
}

pub struct Scene {
    /// Les 4 crêtes, de l'arrière vers l'avant
    pub cretes: Vec<Tampon>,
    pub etoiles: Tampon,
    pub etoiles_scintillantes: Tampon,
    pub lune: Tampon,
    /// Coin haut-gauche de l'image de la lune, et sa taille (pixels de la scène)
    pub lune_x: f32,
    pub lune_y: f32,
    pub lune_taille: f32,
    pub nappes: Vec<Nappe>,
}

impl Scene {
    /// Convertit pour Slint (sur le thread de l'interface).
    pub fn vers_slint(self) -> SceneSlint {
        SceneSlint {
            cretes: self.cretes.into_iter().map(Image::from_rgba8).collect(),
            etoiles: Image::from_rgba8(self.etoiles),
            etoiles_scintillantes: Image::from_rgba8(self.etoiles_scintillantes),
            lune: Image::from_rgba8(self.lune),
            lune_x: self.lune_x,
            lune_y: self.lune_y,
            lune_taille: self.lune_taille,
            nappes: self
                .nappes
                .into_iter()
                .map(|n| crate::ScenePuff {
                    image: Image::from_rgba8(n.image),
                    x: n.x,
                    y: n.y,
                    w: n.w,
                    h: n.h,
                    layer: n.couche,
                    period: n.periode_ms,
                    phase: n.phase,
                    opacity: n.opacite,
                })
                .collect(),
        }
    }
}

pub struct SceneSlint {
    pub cretes: Vec<Image>,
    pub etoiles: Image,
    pub etoiles_scintillantes: Image,
    pub lune: Image,
    pub lune_x: f32,
    pub lune_y: f32,
    pub lune_taille: f32,
    pub nappes: Vec<crate::ScenePuff>,
}

/// Générateur pseudo-aléatoire minimal (xorshift64*), identique au site.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }
    fn float(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }
    fn range(&mut self, lo: i32, hi: i32) -> i32 {
        lo + (self.next() % (hi - lo) as u64) as i32
    }
}

/// Masque blanc : pixels allumés avec une opacité donnée.
struct Masque {
    l: i32,
    h: i32,
    alpha: Vec<u8>,
}

/// Réglage du flou : 0 = pixel art net. Sinon, le masque est agrandi (`facteur`), entouré d'une
/// marge transparente (`marge`, en pixels agrandis) puis flouté (`sigma`, en pixels agrandis).
#[derive(Clone, Copy)]
struct Flou {
    sigma: f32,
}

impl Flou {
    fn actif(self) -> bool {
        self.sigma > 0.0
    }
    fn marge(self) -> i32 {
        (self.sigma * 3.0).ceil() as i32
    }
}

impl Masque {
    fn new(l: i32, h: i32) -> Self {
        Masque { l, h, alpha: vec![0; (l * h) as usize] }
    }
    fn remplir(&mut self, x: i32, y: i32, l: i32, h: i32, a: u8) {
        for yy in y.max(0)..(y + h).min(self.h) {
            for xx in x.max(0)..(x + l).min(self.l) {
                self.alpha[(yy * self.l + xx) as usize] = a;
            }
        }
    }
    fn tampon(&self) -> Tampon {
        let mut tampon = Tampon::new(self.l as u32, self.h as u32);
        for (p, a) in tampon.make_mut_slice().iter_mut().zip(&self.alpha) {
            *p = Rgba8Pixel { r: 255, g: 255, b: 255, a: *a };
        }
        tampon
    }
    /// Version floutée : agrandie `facteur` fois, avec `marge` pixels (agrandis) de chaque côté.
    fn tampon_flou(&self, flou: Flou, facteur: i32, marge: i32) -> Tampon {
        if !flou.actif() {
            return self.tampon();
        }
        let (l, h) = (self.l * facteur + 2 * marge, self.h * facteur + 2 * marge);
        let mut img = image::GrayImage::new(l as u32, h as u32);
        for y in 0..self.h {
            for x in 0..self.l {
                let a = self.alpha[(y * self.l + x) as usize];
                if a == 0 {
                    continue;
                }
                for dy in 0..facteur {
                    for dx in 0..facteur {
                        img.put_pixel((marge + x * facteur + dx) as u32, (marge + y * facteur + dy) as u32, image::Luma([a]));
                    }
                }
            }
        }
        let floue = image::imageops::blur(&img, flou.sigma);
        let mut tampon = Tampon::new(l as u32, h as u32);
        for (p, a) in tampon.make_mut_slice().iter_mut().zip(floue.as_raw()) {
            *p = Rgba8Pixel { r: 255, g: 255, b: 255, a: *a };
        }
        tampon
    }
}

/// Hauteur de crête (en cases depuis le haut) pour chaque colonne.
fn crete(rng: &mut Rng, base: f32, amp: f32, freq: f32) -> Vec<i32> {
    let (p1, p2, p3) = (rng.float() * 6.3, rng.float() * 6.3, rng.float() * 6.3);
    let mut jitter = 0i32;
    (0..COLS)
        .map(|c| {
            let x = c as f32 * freq;
            let vague = 0.55 * (x + p1).sin() + 0.3 * (x * 2.3 + p2).sin() + 0.15 * (x * 5.1 + p3).sin();
            if rng.float() < 0.35 {
                jitter = (jitter + rng.range(-1, 2)).clamp(-1, 1);
            }
            (base - amp * vague).round() as i32 + jitter
        })
        .collect()
}

/// Une nappe de brume : quelques rangées empilées qui rétrécissent vers le haut.
fn nappe(rng: &mut Rng) -> Masque {
    let rangees = rng.range(3, 6);
    let base = rng.range(16, 44);
    let mut m = Masque::new(base, rangees);
    let (mut gauche, mut droite) = (0, base);
    for r in 0..rangees {
        m.remplir(gauche, rangees - 1 - r, droite - gauche, 1, 255);
        gauche += rng.range(1, 5);
        droite -= rng.range(1, 5);
        if droite - gauche < 4 {
            break;
        }
    }
    m
}

/// Génère le paysage. `niveau_flou` : 0 (net) à 10 (très flou).
pub fn generer(niveau_flou: u32) -> Scene {
    // Les masques floutés sont agrandis à 2 px de la scène par pixel : sigma en pixels agrandis
    let flou = Flou { sigma: niveau_flou.min(10) as f32 * 0.85 };
    // Échelle des calques floutés : une case (8 px de scène) = 4 pixels agrandis
    const F: i32 = 4;
    let marge = flou.marge();
    // marge en pixels de la scène (1 pixel agrandi = 2 px de scène)
    let marge_scene = (marge * 2) as f32;

    let mut rng = Rng(7 | 1);

    // Étoiles (4 pixels d'image par case, comme le site : 2 px pour les petites, 4 px pour les grandes)
    let (mut etoiles, mut scintillantes) = (Masque::new(COLS * 4, ROWS * 4), Masque::new(COLS * 4, ROWS * 4));
    for _ in 0..90 {
        let (x, y) = (rng.range(0, COLS) * 4, rng.range(0, 26) * 4);
        let taille = if rng.float() < 0.2 { 2 } else { 1 };
        if rng.float() < 0.3 {
            scintillantes.remplir(x, y, taille, taille, 255);
        } else {
            etoiles.remplir(x, y, taille, taille, 140);
        }
    }
    // Les étoiles floutées restent visibles : flou plus léger, sinon elles disparaissent
    let flou_etoiles = Flou { sigma: flou.sigma * 0.5 };

    // Lune : un disque de cases, quelques cratères plus sombres (moins opaques)
    let cases = LUNE_R / P;
    let cote = cases * 2 + 1;
    let mut lune = Masque::new(cote, cote);
    for cy in -cases..=cases {
        for cx in -cases..=cases {
            if cx * cx + cy * cy <= cases * cases {
                let cratere = [(-2, -1), (-1, -1), (-2, 0), (2, 2), (1, -3), (3, -1)]
                    .iter()
                    .any(|&(x, y)| cx == x * cases / 5 && cy == y * cases / 5);
                lune.remplir(cx + cases, cy + cases, 1, 1, if cratere { 200 } else { 255 });
            }
        }
    }
    let (mx, my) = ((LUNE_X * W as f32) as i32 / P * P, 13 * P);

    // Crêtes de l'arrière vers l'avant, avec des sapins sur les deux plus proches
    let couches: [(f32, f32, f32); 4] = [(30.0, 8.0, 0.045), (37.0, 7.0, 0.07), (44.0, 5.0, 0.09), (50.0, 3.0, 0.12)];
    let mut cretes = Vec::new();
    let mut nappes = Vec::new();
    for (i, (base, amp, freq)) in couches.iter().enumerate() {
        let hauteurs = crete(&mut rng, *base, *amp, *freq);
        let mut m = Masque::new(COLS, ROWS);
        for (c, h) in hauteurs.iter().enumerate() {
            m.remplir(c as i32, *h, 1, ROWS - h, 255);
        }
        if i >= 2 {
            let mut c = rng.range(2, 10);
            while c < COLS - 3 {
                let sommet = hauteurs[c as usize];
                let grand = rng.range(3, if i == 3 { 7 } else { 5 });
                for rangee in 0..grand {
                    let demi = (rangee + 1) / 2;
                    m.remplir(c - demi, sommet - grand + rangee, demi * 2 + 1, 1, 255);
                }
                c += rng.range(4, 16);
            }
        }
        // les crêtes couvrent toute la scène : pas de marge (les bords sont hors champ)
        cretes.push(m.tampon_flou(flou, F, 0));

        // Nappes de brume, de plus en plus visibles vers l'avant
        let ligne = *base as i32;
        let opacite = 0.05 + 0.025 * i as f32;
        for _ in 0..(3 + i as i32) {
            let y = rng.range(ligne - 6, ligne + 2) * P;
            let x = rng.range(0, COLS) * P;
            let duree = rng.range(150, 320);
            let retard = rng.range(0, duree);
            let op = opacite * (0.7 + rng.float() * 0.6);
            let forme = nappe(&mut rng);
            let (dx, dl) = if flou.actif() { (marge_scene, 2.0 * marge_scene) } else { (0.0, 0.0) };
            nappes.push(Nappe {
                image: forme.tampon_flou(flou, F, marge),
                x: x as f32 - dx,
                y: y as f32 - dx,
                w: (forme.l * P) as f32 + dl,
                h: (forme.h * P) as f32 + dl,
                couche: i as i32,
                periode_ms: (duree * 1000) as i64,
                phase: retard as f32 / duree as f32,
                opacite: op,
            });
        }
    }

    let marge_lune = if flou.actif() { marge_scene } else { 0.0 };
    Scene {
        cretes,
        etoiles: etoiles.tampon_flou(flou_etoiles, 1, 0),
        etoiles_scintillantes: scintillantes.tampon_flou(flou_etoiles, 1, 0),
        lune: lune.tampon_flou(flou, F, marge),
        lune_x: (mx - cases * P - P / 2) as f32 - marge_lune,
        lune_y: (my - cases * P - P / 2) as f32 - marge_lune,
        lune_taille: (cote * P) as f32 + 2.0 * marge_lune,
        nappes,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn flou_garde_les_dimensions_utiles() {
        let net = super::generer(0);
        let flou = super::generer(6);
        assert_eq!(net.cretes.len(), 4);
        assert_eq!(net.nappes.len(), flou.nappes.len());
        // la lune floutée est plus grande (marge) mais reste centrée au même endroit
        let centre = |s: &super::Scene| (s.lune_x + s.lune_taille / 2.0, s.lune_y + s.lune_taille / 2.0);
        assert!((centre(&net).0 - centre(&flou).0).abs() < 0.01);
        assert!(flou.lune_taille > net.lune_taille);
    }
}

/// Paysage du menu principal de Misty Client (le mod) : les calques du thème nuit, nets, en PNG
/// dans ses ressources. Chaque calque est suivi de son reflet, pour défiler en boucle sans
/// raccord visible : `cargo test exporter_paysage_du_mod -- --ignored`.
#[cfg(test)]
mod export_mod {
    use super::*;

    fn png(tampon: &Tampon, couleur: [u8; 3], reflet: bool) -> image::RgbaImage {
        let (l, h) = (tampon.width(), tampon.height());
        let source = tampon.as_slice();
        let largeur = if reflet { l * 2 } else { l };
        let mut img = image::RgbaImage::new(largeur, h);
        for y in 0..h {
            for x in 0..largeur {
                let sx = if x < l { x } else { 2 * l - 1 - x };
                let a = source[(y * l + sx) as usize].a;
                img.put_pixel(x, y, image::Rgba([couleur[0], couleur[1], couleur[2], a]));
            }
        }
        img
    }

    #[test]
    #[ignore]
    fn exporter_paysage_du_mod() {
        let scene = generer(0);
        let dossier = std::path::Path::new("../misty-client/src/main/resources/assets/misty/textures/gui/scene");
        std::fs::create_dir_all(dossier).unwrap();
        // couleurs du thème nuit du launcher (ui/scene.slint)
        let cretes = [[0x24, 0x34, 0x5A], [0x1A, 0x27, 0x48], [0x12, 0x1C, 0x36], [0x0D, 0x15, 0x28]];
        for (i, c) in scene.cretes.iter().enumerate() {
            png(c, cretes[i], true).save(dossier.join(format!("ridge{i}.png"))).unwrap();
        }
        png(&scene.etoiles, [0xDB, 0xE7, 0xFF], true).save(dossier.join("stars.png")).unwrap();
        png(&scene.etoiles_scintillantes, [0xFF, 0xFF, 0xFF], true).save(dossier.join("twinkle.png")).unwrap();
        png(&scene.lune, [0xEA, 0xF1, 0xFF], false).save(dossier.join("moon.png")).unwrap();

        // Le même paysage en données, pour l'écran de chargement : à ce moment-là, le jeu ne peut pas
        // encore lire les textures du mod, alors il est dessiné en rectangles (voir SceneData.java).
        let segments = |t: &Tampon| -> String {
            let (l, h) = (t.width() as usize, t.height() as usize);
            let px = t.as_slice();
            (0..l)
                .map(|x| {
                    let mut col = String::new();
                    let mut y = 0;
                    while y < h {
                        if px[y * l + x].a > 0 {
                            let debut = y;
                            while y < h && px[y * l + x].a > 0 {
                                y += 1;
                            }
                            col.push((b'A' + debut as u8) as char);
                            col.push((b'A' + y as u8) as char);
                        } else {
                            y += 1;
                        }
                    }
                    col
                })
                .collect::<Vec<_>>()
                .join(",")
        };
        let points = |t: &Tampon| -> String {
            let (l, h) = (t.width() as usize, t.height() as usize);
            let px = t.as_slice();
            let mut v = Vec::new();
            for y in 0..h {
                for x in 0..l {
                    if px[y * l + x].a > 0 {
                        v.push(format!("{x}.{y}"));
                    }
                }
            }
            v.join(" ")
        };
        let grille = |t: &Tampon| -> String {
            t.as_slice().iter().map(|p| if p.a == 0 { '0' } else if p.a < 255 { '2' } else { '1' }).collect()
        };
        let java = format!(
            "package online.mistylauncher.client.menu;

/**
 * The night landscape as data, for the loading screen (the textures of
 * the mod cannot be read while the game loads its resources). Written by the launcher:
 * {{@code misty-rust/src/ui/scene.rs}}, {{@code exporter_paysage_du_mod}}. Do not edit by hand.
 */
final class SceneData {{
	/** Ridges (far to near): per column of 200, the filled rows as pairs of letters ('A' = row 0, end excluded). */
	static final String[] RIDGES = {{
{}
	}};
	/** Stars (800 × 220 grid): \"x.y\" of each lit pixel. */
	static final String STARS = \"{}\";
	static final String TWINKLE = \"{}\";
	/** Moon, 11 × 11: 0 empty, 1 lit, 2 crater. */
	static final String MOON = \"{}\";

	private SceneData() {{
	}}
}}
",
            scene.cretes.iter().map(|c| format!("		\"{}\"", segments(c))).collect::<Vec<_>>().join(",
"),
            points(&scene.etoiles),
            points(&scene.etoiles_scintillantes),
            grille(&scene.lune)
        );
        // les données contiennent parfois un antislash ('A' + 27) : échappé pour la chaîne Java
        std::fs::write("../misty-client/src/main/java/online/mistylauncher/client/menu/SceneData.java", java.replace('\\', "\\\\")).unwrap();
        println!(
            "crêtes {}x{} · étoiles {}x{} · lune {}x{} à ({}, {}) taille {}",
            scene.cretes[0].width(),
            scene.cretes[0].height(),
            scene.etoiles.width(),
            scene.etoiles.height(),
            scene.lune.width(),
            scene.lune.height(),
            scene.lune_x,
            scene.lune_y,
            scene.lune_taille
        );
    }
}
