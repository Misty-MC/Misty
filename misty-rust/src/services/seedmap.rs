//! Visualiseur de seed : biomes et structures d'un monde Minecraft, calculés par cubiomes (MIT, dossier vendor/).
//! Le même algorithme que le jeu : la carte montre ce que le monde contiendra, sans avoir à le générer.

use image::{Rgba, RgbaImage};
use std::ffi::{c_char, CStr};

extern "C" {
    fn misty_biomes(mc: i32, dim: i32, seed: i64, scale: i32, x: i32, z: i32, sx: i32, sz: i32, sortie: *mut i32) -> i32;
    fn misty_couleurs(sortie: *mut u8);
    fn misty_nom_biome(mc: i32, id: i32) -> *const c_char;
    fn misty_spawn(mc: i32, seed: i64, x: *mut i32, z: *mut i32);
    fn misty_structures(mc: i32, dim: i32, seed: i64, type_: i32, x0: i32, z0: i32, x1: i32, z1: i32, sortie: *mut i32, max: i32) -> i32;
}

/// Versions proposées : (libellé, identifiant cubiomes). La plus récente d'abord.
pub const VERSIONS: &[(&str, i32)] = &[
    ("1.21.4+ (latest)", 28),
    ("1.21.2 - 1.21.3", 27),
    ("1.21 - 1.21.1", 26),
    ("1.20", 25),
    ("1.19.3 - 1.19.4", 24),
    ("1.19 - 1.19.2", 23),
    ("1.18", 22),
    ("1.17", 21),
    ("1.16.2+", 20),
    ("1.16.1", 19),
    ("1.15", 18),
    ("1.14", 17),
    ("1.13", 16),
    ("1.12", 15),
    ("1.11", 14),
    ("1.10", 13),
    ("1.9", 12),
    ("1.8", 11),
    ("1.7", 10),
    ("1.6", 9),
    ("1.5", 8),
    ("1.4", 7),
    ("1.3", 6),
    ("1.2", 5),
    ("1.1", 4),
    ("1.0", 3),
];

/// Dimensions : (clé de traduction, numéro cubiomes).
pub const DIMENSIONS: &[(&str, i32)] = &[("Tools_Overworld", 0), ("Tools_Nether", -1), ("Tools_End", 1)];

/// Une sorte de structure à afficher.
pub struct Genre {
    pub cle: &'static str,
    pub couleur: [u8; 3],
    pub dimension: i32,
    /// Types cubiomes regroupés sous ce genre
    pub types: &'static [i32],
}

pub const GENRES: &[Genre] = &[
    Genre { cle: "village", couleur: [230, 160, 60], dimension: 0, types: &[5] },
    Genre { cle: "outpost", couleur: [190, 60, 60], dimension: 0, types: &[10] },
    Genre { cle: "mansion", couleur: [120, 70, 40], dimension: 0, types: &[9] },
    Genre { cle: "monument", couleur: [40, 190, 190], dimension: 0, types: &[8] },
    Genre { cle: "temple", couleur: [240, 220, 90], dimension: 0, types: &[1, 2, 3, 4] },
    Genre { cle: "portal", couleur: [170, 90, 230], dimension: 0, types: &[11] },
    Genre { cle: "city", couleur: [40, 90, 120], dimension: 0, types: &[13] },
    Genre { cle: "trial", couleur: [220, 130, 70], dimension: 0, types: &[24] },
    Genre { cle: "fortress", couleur: [200, 50, 50], dimension: -1, types: &[18] },
    Genre { cle: "bastion", couleur: [90, 90, 100], dimension: -1, types: &[19] },
    Genre { cle: "portal_n", couleur: [170, 90, 230], dimension: -1, types: &[12] },
    Genre { cle: "endcity", couleur: [220, 120, 220], dimension: 1, types: &[20] },
];

/// Seed saisie : un nombre, ou un texte (le jeu en fait un nombre avec hashCode de Java). Vide : au hasard.
pub fn lire_seed(texte: &str) -> i64 {
    let t = texte.trim();
    if t.is_empty() {
        return rand::random::<i32>() as i64 * 7919 + rand::random::<i32>() as i64;
    }
    if let Ok(n) = t.parse::<i64>() {
        return n;
    }
    let mut h: i32 = 0;
    for u in t.encode_utf16() {
        h = h.wrapping_mul(31).wrapping_add(u as i32);
    }
    h as i64
}

/// Ce que l'on veut voir : la zone, la version, la dimension, les structures.
#[derive(Clone)]
pub struct Demande {
    pub mc: i32,
    pub dim: i32,
    pub seed: i64,
    /// centre de la carte, en blocs
    pub cx: f64,
    pub cz: f64,
    /// niveau de zoom (voir NIVEAUX)
    pub niveau: usize,
    pub genres: Vec<usize>,
    pub largeur: u32,
    pub hauteur: u32,
    pub spawn: bool,
}

/// Zoom : (échelle cubiomes = blocs par case, pixels par case). Plus l'échelle est grande, plus on voit large.
pub const NIVEAUX: &[(i32, u32)] = &[(1, 4), (4, 2), (16, 2), (64, 2), (256, 2)];

impl Demande {
    /// Blocs par pixel de l'image.
    pub fn blocs_par_pixel(&self) -> f64 {
        let (echelle, px) = NIVEAUX[self.niveau];
        echelle as f64 / px as f64
    }
}

/// La carte calculée, avec de quoi retrouver le biome sous un pixel.
pub struct Rendu {
    pub image: RgbaImage,
    biomes: Vec<i32>,
    cases_x: u32,
    x0: f64,
    z0: f64,
    echelle: i32,
    px: u32,
    mc: i32,
}

impl Rendu {
    /// Biome et position en blocs sous le pixel (x, y) de l'image.
    pub fn sous_pixel(&self, x: f32, y: f32) -> Option<(i32, i32, String)> {
        let (cx, cz) = ((x as u32 / self.px) as usize, (y as u32 / self.px) as usize);
        let id = *self.biomes.get(cz * self.cases_x as usize + cx)?;
        let bx = (self.x0 + x as f64 * self.echelle as f64 / self.px as f64).floor() as i32;
        let bz = (self.z0 + y as f64 * self.echelle as f64 / self.px as f64).floor() as i32;
        // SAFETY : misty_nom_biome renvoie une chaîne C statique (ou NULL)
        let nom = unsafe {
            let p = misty_nom_biome(self.mc, id);
            if p.is_null() { String::new() } else { CStr::from_ptr(p).to_string_lossy().replace('_', " ") }
        };
        Some((bx, bz, nom))
    }
}

pub fn position_spawn(mc: i32, seed: i64) -> (i32, i32) {
    let (mut x, mut z) = (0, 0);
    // SAFETY : les pointeurs désignent des entiers valides
    unsafe { misty_spawn(mc, seed, &mut x, &mut z) };
    (x, z)
}

fn palette() -> Vec<[u8; 3]> {
    let mut brut = vec![0u8; 256 * 3];
    // SAFETY : le tampon fait bien 256 * 3 octets
    unsafe { misty_couleurs(brut.as_mut_ptr()) };
    brut.chunks(3).map(|c| [c[0], c[1], c[2]]).collect()
}

fn marqueur(image: &mut RgbaImage, x: i64, y: i64, couleur: [u8; 3]) {
    for dy in -5i64..=5 {
        for dx in -5i64..=5 {
            let (px, py) = (x + dx, y + dy);
            if px < 0 || py < 0 || px >= image.width() as i64 || py >= image.height() as i64 {
                continue;
            }
            let bord = dx.abs() == 5 || dy.abs() == 5;
            let c = if bord { Rgba([20, 20, 30, 255]) } else if dx.abs() == 4 || dy.abs() == 4 { Rgba([255, 255, 255, 255]) } else { Rgba([couleur[0], couleur[1], couleur[2], 255]) };
            image.put_pixel(px as u32, py as u32, c);
        }
    }
}

/// Calcule la carte (long : à appeler hors du fil de l'interface).
pub fn rendre(d: &Demande) -> Option<Rendu> {
    let (echelle, px) = NIVEAUX[d.niveau];
    let cases_x = d.largeur / px;
    let cases_z = d.hauteur / px;
    // case en haut à gauche, en unités d'échelle
    let bpp = echelle as f64 / px as f64;
    let x0 = d.cx - d.largeur as f64 / 2.0 * bpp;
    let z0 = d.cz - d.hauteur as f64 / 2.0 * bpp;
    let case_x = (x0 / echelle as f64).floor() as i32;
    let case_z = (z0 / echelle as f64).floor() as i32;
    let x0 = case_x as f64 * echelle as f64;
    let z0 = case_z as f64 * echelle as f64;

    let mut biomes = vec![0i32; (cases_x * cases_z) as usize];
    // SAFETY : le tampon contient cases_x * cases_z entiers, comme demandé
    let erreur = unsafe { misty_biomes(d.mc, d.dim, d.seed, echelle, case_x, case_z, cases_x as i32, cases_z as i32, biomes.as_mut_ptr()) };
    if erreur != 0 {
        return None;
    }

    let couleurs = palette();
    let mut image = RgbaImage::new(cases_x * px, cases_z * px);
    for cz in 0..cases_z {
        for cx in 0..cases_x {
            let id = biomes[(cz * cases_x + cx) as usize];
            let c = couleurs.get(id.max(0) as usize).copied().unwrap_or([60, 60, 60]);
            let pixel = Rgba([c[0], c[1], c[2], 255]);
            for dy in 0..px {
                for dx in 0..px {
                    image.put_pixel(cx * px + dx, cz * px + dy, pixel);
                }
            }
        }
    }

    // axes x = 0 et z = 0
    let vers_pixel = |bx: f64, bz: f64| -> (i64, i64) { (((bx - x0) / bpp).round() as i64, ((bz - z0) / bpp).round() as i64) };
    let (ox, oz) = vers_pixel(0.0, 0.0);
    for y in 0..image.height() {
        if ox >= 0 && (ox as u32) < image.width() {
            blanchir(&mut image, ox as u32, y);
        }
    }
    for x in 0..image.width() {
        if oz >= 0 && (oz as u32) < image.height() {
            blanchir(&mut image, x, oz as u32);
        }
    }

    // structures
    let (x1, z1) = (x0 + image.width() as f64 * bpp, z0 + image.height() as f64 * bpp);
    for &i in &d.genres {
        let Some(genre) = GENRES.get(i) else { continue };
        if genre.dimension != d.dim {
            continue;
        }
        for &type_ in genre.types {
            let mut sortie = vec![0i32; 2 * 600];
            // SAFETY : la sortie peut contenir 600 positions
            let n = unsafe { misty_structures(d.mc, d.dim, d.seed, type_, x0 as i32, z0 as i32, x1 as i32, z1 as i32, sortie.as_mut_ptr(), 600) };
            for k in 0..n.max(0) as usize {
                let (x, y) = vers_pixel(sortie[k * 2] as f64, sortie[k * 2 + 1] as f64);
                marqueur(&mut image, x, y, genre.couleur);
            }
        }
    }

    // point d'apparition (surface seulement)
    if d.spawn && d.dim == 0 {
        let (sx, sz) = position_spawn(d.mc, d.seed);
        let (x, y) = vers_pixel(sx as f64, sz as f64);
        marqueur(&mut image, x, y, [255, 60, 60]);
    }

    Some(Rendu { image, biomes, cases_x, x0, z0, echelle, px, mc: d.mc })
}

fn blanchir(image: &mut RgbaImage, x: u32, y: u32) {
    let p = image.get_pixel_mut(x, y);
    for i in 0..3 {
        p[i] = ((p[i] as u16 * 6 + 255 * 4) / 10) as u8;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seed_texte_comme_java() {
        assert_eq!(lire_seed("12345"), 12345);
        assert_eq!(lire_seed("-5"), -5);
        assert_eq!(lire_seed("Hello"), 69609650); // "Hello".hashCode() en Java
    }

    #[test]
    fn carte_de_base() {
        let d = Demande { mc: 28, dim: 0, seed: 1, cx: 0.0, cz: 0.0, niveau: 2, genres: vec![0, 1], largeur: 480, hauteur: 220, spawn: true };
        let r = rendre(&d).expect("rendu");
        assert_eq!(r.image.width(), 480);
        let (_, _, nom) = r.sous_pixel(240.0, 110.0).unwrap();
        println!("biome au centre : {nom}");
        assert!(!nom.is_empty());
        r.image.save(std::env::temp_dir().join("misty-seed.png")).unwrap();
    }
}
