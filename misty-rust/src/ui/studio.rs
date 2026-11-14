//! Rendu en direct du studio (onglet Skin) : un fil dédié dessine le personnage à la demande de
//! l'interface (une demande par image tant que le studio est visible), avec l'angle de la caméra et
//! l'animation choisis. Si un dessin est déjà en cours, la demande est ignorée : on ne prend jamais
//! de retard, on saute simplement une image.

use super::App;
use crate::services::skins::{self, Anim};
use crate::SkinState;
use image::RgbaImage;
use slint::{ComponentHandle, Image, Rgba8Pixel, SharedPixelBuffer};
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::Instant;

/// Ce que porte le personnage dessiné.
#[derive(Default)]
struct Modele {
    skin: Option<Arc<RgbaImage>>,
    cape: Option<Arc<RgbaImage>>,
    fin: bool,
    elytres: bool,
}

static MODELE: Mutex<Modele> = Mutex::new(Modele { skin: None, cape: None, fin: false, elytres: false });
static OCCUPE: AtomicBool = AtomicBool::new(false);
static DEMANDES: OnceLock<mpsc::Sender<(f32, f32, i32)>> = OnceLock::new();
/// Partie du corps visible en chaque pixel de la dernière image (voir skins::rendre_live_parties).
static PARTIES: Mutex<Vec<u8>> = Mutex::new(Vec::new());

/// Parties teintées au prochain rendu : celles de l'emplacement que la souris survole.
static SURVOL: Mutex<Vec<u8>> = Mutex::new(Vec::new());

/// Emplacement de cosmétique d'une partie du corps : 0 cape (dos, corps, cape), 1 ailes, 2 chapeau.
pub fn emplacement(partie: u8) -> Option<i32> {
    use skins::partie::*;
    match partie {
        TETE => Some(2),
        ELYTRES => Some(1),
        CAPE => Some(0),
        _ => None,
    }
}

/// La souris survole le point (u, v) du personnage : les parties de cet emplacement seront
/// teintées au prochain rendu. Renvoie l'emplacement (-1 : aucun).
pub fn survoler(u: f32, v: f32) -> i32 {
    let place = partie(u, v).and_then(emplacement).unwrap_or(-1);
    let parties: Vec<u8> = (0..=skins::partie::ELYTRES).filter(|&p| emplacement(p) == Some(place)).collect();
    if let Ok(mut s) = SURVOL.lock() {
        *s = parties;
    }
    place
}

/// Partie du corps sous un point de l'image affichée (u, v entre 0 et 1), s'il y en a une.
pub fn partie(u: f32, v: f32) -> Option<u8> {
    if !(0.0..1.0).contains(&u) || !(0.0..1.0).contains(&v) {
        return None;
    }
    let cote = skins::LIVE as f32;
    let (x, y) = ((u * cote) as usize, (v * cote) as usize);
    let carte = PARTIES.lock().ok()?;
    carte.get(y * skins::LIVE as usize + x).copied().filter(|&p| p != skins::partie::AUCUNE)
}

pub fn installer(app: &Rc<App>) {
    let (tx, rx) = mpsc::channel::<(f32, f32, i32)>();
    let fenetre = app.ui.as_weak();
    std::thread::Builder::new()
        .name("studio".into())
        .spawn(move || {
            let debut = Instant::now();
            while let Ok(mut demande) = rx.recv() {
                // seulement la plus récente
                while let Ok(d) = rx.try_recv() {
                    demande = d;
                }
                let (skin, cape, fin, elytres) = {
                    let m = MODELE.lock().unwrap();
                    (m.skin.clone(), m.cape.clone(), m.fin, m.elytres)
                };
                let (yaw, pitch, anim) = demande;
                let surbrillance = SURVOL.lock().map(|s| s.clone()).unwrap_or_default();
                let rendu = skin.and_then(|skin| {
                    skins::rendre_live_parties(&skin, cape.as_deref(), fin, elytres, Anim::depuis(anim), debut.elapsed().as_secs_f32(), yaw.to_radians(), pitch.to_radians(), &surbrillance)
                });
                let Some((image, carte)) = rendu else {
                    OCCUPE.store(false, Ordering::SeqCst);
                    continue;
                };
                if let Ok(mut p) = PARTIES.lock() {
                    *p = carte;
                }
                let tampon = SharedPixelBuffer::<Rgba8Pixel>::clone_from_slice(image.as_raw(), image.width(), image.height());
                let envoye = fenetre.upgrade_in_event_loop(move |ui| {
                    let st = ui.global::<SkinState>();
                    st.set_live(Image::from_rgba8(tampon));
                    st.set_live_ready(true);
                    OCCUPE.store(false, Ordering::SeqCst);
                });
                if envoye.is_err() {
                    break; // fenêtre fermée
                }
            }
        })
        .expect("fil du studio");
    let _ = DEMANDES.set(tx);

    app.ui.global::<SkinState>().on_render_frame(|yaw, pitch, anim| {
        if OCCUPE.swap(true, Ordering::SeqCst) {
            return; // image précédente pas encore finie : on saute celle-ci
        }
        if let Some(tx) = DEMANDES.get() {
            if tx.send((yaw, pitch, anim)).is_err() {
                OCCUPE.store(false, Ordering::SeqCst);
            }
        }
    });
}

/// Change ce que porte le personnage (skin, bras fins, cape ou élytres).
pub fn definir(png: &[u8], fin: bool, cape: Option<&[u8]>, elytres: bool) {
    let skin = image::load_from_memory(png).ok().map(|i| Arc::new(i.to_rgba8()));
    let cape = cape.and_then(|c| image::load_from_memory(c).ok()).map(|i| Arc::new(i.to_rgba8()));
    let mut m = MODELE.lock().unwrap();
    if skin.is_some() {
        m.skin = skin;
    }
    m.cape = cape;
    m.fin = fin;
    m.elytres = elytres;
}
