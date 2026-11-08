//! Paysage du site misty-launcher.online (montagnes, sapins, lune, étoiles, nappes de brume),
//! repris en fond du launcher.
//!
//! Même générateur que le site (même graine = même paysage), mais au lieu d'un SVG on produit des
//! masques blancs en pixels : Slint les agrandit sans lissage (pixel art net) et les colore selon le
//! thème (nuit, brume, Halloween, Noël). Les animations (lune qui flotte, étoiles qui scintillent,
//! brume qui dérive) sont faites côté Slint.

use crate::ScenePuff;
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

pub struct Scene {
    /// Les 4 crêtes, de l'arrière vers l'avant (1 pixel d'image = 1 case de 8 px)
    pub cretes: Vec<Image>,
    pub etoiles: Image,
    pub etoiles_scintillantes: Image,
    pub lune: Image,
    /// Coin haut-gauche de l'image de la lune, et sa taille (pixels de la scène)
    pub lune_x: f32,
    pub lune_y: f32,
    pub lune_taille: f32,
    pub nappes: Vec<ScenePuff>,
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
    fn image(&self) -> Image {
        let mut tampon = SharedPixelBuffer::<Rgba8Pixel>::new(self.l as u32, self.h as u32);
        for (p, a) in tampon.make_mut_slice().iter_mut().zip(&self.alpha) {
            *p = Rgba8Pixel { r: 255, g: 255, b: 255, a: *a };
        }
        Image::from_rgba8(tampon)
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

pub fn generer() -> Scene {
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
        cretes.push(m.image());

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
            nappes.push(ScenePuff {
                image: forme.image(),
                x: x as f32,
                y: y as f32,
                w: (forme.l * P) as f32,
                h: (forme.h * P) as f32,
                layer: i as i32,
                period: (duree * 1000) as i64,
                phase: retard as f32 / duree as f32,
                opacity: op,
            });
        }
    }

    Scene {
        cretes,
        etoiles: etoiles.image(),
        etoiles_scintillantes: scintillantes.image(),
        lune: lune.image(),
        lune_x: (mx - cases * P - P / 2) as f32,
        lune_y: (my - cases * P - P / 2) as f32,
        lune_taille: (cote * P) as f32,
        nappes,
    }
}
