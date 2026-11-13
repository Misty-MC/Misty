//! Images : logo nuage, icônes distantes (avec cache), images locales.

use super::App;
use slint::{Image, Rgba8Pixel, SharedPixelBuffer};
use std::cell::RefCell;
use std::collections::HashMap;
use std::path::Path;

/// Cache des icônes distantes (thread de l'interface seulement).
#[derive(Default)]
pub struct Cache {
    images: RefCell<HashMap<String, Option<Image>>>,
}

/// Le nuage de Misty (vignettes sans image, fond, carte de version).
pub fn logo_nuage() -> Image {
    decoder(include_bytes!("../../assets/logo-nuage.png"), 0).map(Image::from_rgba8).unwrap_or_default()
}

/// Décode une image (fichier ou octets) en tampon de pixels, réduite à `max` pixels de large.
pub fn decoder(octets: &[u8], max: u32) -> Option<SharedPixelBuffer<Rgba8Pixel>> {
    let img = image::load_from_memory(octets).ok()?;
    let img = if max > 0 && img.width() > max {
        img.resize(max, max * img.height() / img.width().max(1), image::imageops::FilterType::Triangle)
    } else {
        img
    };
    Some(tampon(&img.to_rgba8()))
}

/// Logo de l'écran de chargement, comme celui de Misty Client (MistyLoading.java) : le nuage en
/// pixel art (20 × 13, chaque pixel en fait 2) puis « MISTY » dans une police 5 × 7 avec son
/// ombre. Une unité = un pixel du logo : 40 × 40, affiché agrandi sans lissage.
pub fn logo_chargement() -> Image {
    const NUAGE: [&str; 13] = [
        ".......#####........",
        ".....#########......",
        "....###########.....",
        "....############....",
        "..###############...",
        ".#################..",
        "####################",
        "####################",
        "####################",
        ".##################.",
        "....................",
        "..########..######..",
        "....................",
    ];
    const LETTRES: [&str; 5] = [
        "10001110111010110001100011000110001",
        "11111001000010000100001000010011111",
        "01111100001000001110000010000111110",
        "11111001000010000100001000010000100",
        "10001100010101000100001000010000100",
    ];
    let (blanc, ligne, ardoise) = (image::Rgba([255, 255, 255, 255]), image::Rgba([0xDC, 0xE6, 0xFF, 255]), image::Rgba([0x3C, 0x54, 0x86, 255]));
    let mut img = image::RgbaImage::new(40, 40);
    for (rang, motif) in NUAGE.iter().enumerate() {
        for (col, c) in motif.chars().enumerate() {
            if c == '#' {
                let couleur = if rang > 10 { ligne } else { blanc };
                for (dx, dy) in [(0, 0), (1, 0), (0, 1), (1, 1)] {
                    img.put_pixel(col as u32 * 2 + dx, rang as u32 * 2 + dy, couleur);
                }
            }
        }
    }
    // nom centré sous le nuage (33 unités de large), 6 unités plus bas ; ombre puis pixel, dans
    // le même ordre que le client
    let (x0, y0) = (3u32, 26 + 6);
    for (l, lettre) in LETTRES.iter().enumerate() {
        for (i, c) in lettre.chars().enumerate() {
            if c == '1' {
                let (x, y) = (x0 + l as u32 * 7 + i as u32 % 5, y0 + i as u32 / 5);
                img.put_pixel(x + 1, y + 1, ardoise);
                img.put_pixel(x, y, blanc);
            }
        }
    }
    Image::from_rgba8(tampon(&img))
}

/// Le nuage du bouton JOUER (500 × 116, affiché à moitié pour rester net) : des bosses de tailles
/// différentes fondues les unes dans les autres (union « douce » de disques), un dessous plat, des
/// bords lissés, et un dégradé du blanc en haut au bleu brume en bas.
pub fn nuage_jouer() -> Image {
    const L: u32 = 500;
    const H: u32 = 116;
    // (centre x, centre y, rayon) : grosse bosse au milieu, plus petites sur les côtés
    const BOSSES: [(f32, f32, f32); 7] = [
        (70.0, 84.0, 30.0),
        (128.0, 66.0, 42.0),
        (205.0, 52.0, 50.0),
        (285.0, 42.0, 56.0),
        (360.0, 58.0, 46.0),
        (420.0, 76.0, 34.0),
        (462.0, 90.0, 22.0),
    ];
    const BAS: f32 = H as f32 - 6.0;
    // union douce : plus k est grand, plus les creux entre les bosses sont arrondis
    fn union_douce(a: f32, b: f32, k: f32) -> f32 {
        let h = (0.5 + 0.5 * (b - a) / k).clamp(0.0, 1.0);
        b + (a - b) * h - k * h * (1.0 - h)
    }
    let distance = |x: f32, y: f32| -> f32 {
        let mut d = f32::MAX;
        for (cx, cy, r) in BOSSES {
            let di = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() - r;
            d = if d == f32::MAX { di } else { union_douce(d, di, 22.0) };
        }
        // corps : une longue capsule en bas, puis le dessous coupé à plat
        let (x0, x1, cy, r) = (60.0_f32, 450.0_f32, 86.0_f32, 26.0_f32);
        let px = x.clamp(x0, x1);
        let corps = ((x - px).powi(2) + (y - cy).powi(2)).sqrt() - r;
        d = union_douce(d, corps, 18.0);
        d.max(y - BAS)
    };
    let mut img = image::RgbaImage::new(L, H);
    for y in 0..H {
        let v = y as f32 / H as f32;
        // dégradé : blanc en haut, bleu brume en bas
        let haut = [255.0, 255.0, 255.0];
        let bas = [214.0, 228.0, 248.0];
        let c: Vec<u8> = (0..3).map(|i| (haut[i] + (bas[i] - haut[i]) * v.powf(1.4)).round() as u8).collect();
        for x in 0..L {
            // bord lissé : 4 × 4 échantillons par pixel
            let mut dedans = 0;
            for sy in 0..4 {
                for sx in 0..4 {
                    let d = distance(x as f32 + (sx as f32 + 0.5) / 4.0, y as f32 + (sy as f32 + 0.5) / 4.0);
                    if d <= 0.0 {
                        dedans += 1;
                    }
                }
            }
            if dedans > 0 {
                img.put_pixel(x, y, image::Rgba([c[0], c[1], c[2], (dedans * 255 / 16) as u8]));
            }
        }
    }
    Image::from_rgba8(tampon(&img))
}

pub fn tampon(rgba: &image::RgbaImage) -> SharedPixelBuffer<Rgba8Pixel> {
    SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(rgba.as_raw(), rgba.width(), rgba.height())
}

/// Image locale chargée en mémoire (le fichier n'est pas verrouillé). None si illisible.
pub fn depuis_fichier(chemin: &Path, max: u32) -> Option<Image> {
    let octets = std::fs::read(chemin).ok()?;
    decoder(&octets, max).map(Image::from_rgba8)
}


/// Image distante (icône de projet…), téléchargée en arrière-plan et gardée en cache.
pub async fn depuis_url(app: &App, url: &str) -> Option<Image> {
    if url.is_empty() {
        return None;
    }
    if let Some(img) = app.images.images.borrow().get(url) {
        return img.clone();
    }
    let u = url.to_string();
    let tampon = app
        .bg(async move {
            let reponse = crate::services::http::client().get(&u).send().await.ok()?.error_for_status().ok()?;
            let octets = reponse.bytes().await.ok()?;
            tokio::task::spawn_blocking(move || decoder(&octets, 96)).await.ok()?
        })
        .await;
    let image = tampon.map(Image::from_rgba8);
    app.images.images.borrow_mut().insert(url.to_string(), image.clone());
    image
}

/// Image vide (pour les champs `image` des modèles quand il n'y a rien à afficher).
pub fn vide() -> Image {
    Image::default()
}
