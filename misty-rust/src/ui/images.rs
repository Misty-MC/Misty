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
