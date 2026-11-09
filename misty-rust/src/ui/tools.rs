//! Outils : pour l'instant le visualiseur de seed (biomes et structures d'un monde Minecraft).

use super::{images, lancer, modele, modele_textes, App};
use crate::services::loc;
use crate::services::seedmap::{self, Demande, Rendu};
use crate::{ToolChip, ToolsState};
use slint::{ComponentHandle, Image};
use std::rc::Rc;

/// Taille de la carte affichée (en pixels de l'image).
const LARGEUR: u32 = 960;
const HAUTEUR: u32 = 440;
/// Niveau de zoom au départ (voir seedmap::NIVEAUX)
const NIVEAU_DEPART: usize = 2;

pub struct Etat {
    seed: i64,
    mc: i32,
    dim: i32,
    cx: f64,
    cz: f64,
    niveau: usize,
    /// Genres de structures affichés (indices dans seedmap::GENRES)
    genres: Vec<bool>,
    /// Genres proposés pour la dimension actuelle, dans l'ordre des pastilles
    pastilles: Vec<usize>,
    rendu: Option<Rendu>,
    en_cours: bool,
    a_refaire: bool,
    genere: bool,
    /// Niveau de zoom de la carte affichée (pour l'agrandir tout de suite en attendant le recalcul)
    niveau_rendu: usize,
}

impl Default for Etat {
    fn default() -> Self {
        Etat {
            seed: 0,
            mc: seedmap::VERSIONS[0].1,
            dim: 0,
            cx: 0.0,
            cz: 0.0,
            niveau: NIVEAU_DEPART,
            genres: seedmap::GENRES.iter().map(|g| matches!(g.cle, "village" | "monument" | "mansion" | "fortress" | "endcity")).collect(),
            pastilles: Vec::new(),
            rendu: None,
            en_cours: false,
            a_refaire: false,
            genere: false,
            niveau_rendu: NIVEAU_DEPART,
        }
    }
}

fn s(app: &App) -> ToolsState<'_> {
    app.ui.global::<ToolsState>()
}

pub fn installer(app: &Rc<App>) {
    let st = s(app);
    maj_textes(app);
    maj_pastilles(app);

    let a = app.clone();
    st.on_seed_opened(move || {
        // première ouverture : une seed au hasard, pour ne pas montrer une page vide
        if !a.tools.borrow().genere {
            generer(&a, false);
        }
    });
    let a = app.clone();
    st.on_generate(move || generer(&a, false));
    let a = app.clone();
    st.on_random_seed(move || generer(&a, true));
    let a = app.clone();
    st.on_version_changed(move |i| {
        let mc = seedmap::VERSIONS.get(i.max(0) as usize).map(|v| v.1).unwrap_or(seedmap::VERSIONS[0].1);
        a.tools.borrow_mut().mc = mc;
        redessiner(&a);
    });
    let a = app.clone();
    st.on_dimension_changed(move |i| {
        {
            let mut e = a.tools.borrow_mut();
            e.dim = seedmap::DIMENSIONS.get(i.max(0) as usize).map(|d| d.1).unwrap_or(0);
            e.cx = 0.0;
            e.cz = 0.0;
        }
        maj_pastilles(&a);
        redessiner(&a);
    });
    let a = app.clone();
    st.on_toggle_chip(move |i| {
        {
            let mut e = a.tools.borrow_mut();
            let Some(&genre) = e.pastilles.get(i.max(0) as usize) else { return };
            e.genres[genre] = !e.genres[genre];
        }
        maj_pastilles(&a);
        redessiner(&a);
    });
    let a = app.clone();
    st.on_pan(move |dx, dy| {
        {
            let mut e = a.tools.borrow_mut();
            if !e.genere {
                return;
            }
            let bpp = demande(&e).blocs_par_pixel();
            e.cx -= dx as f64 * LARGEUR as f64 * bpp;
            e.cz -= dy as f64 * HAUTEUR as f64 * bpp;
        }
        redessiner(&a);
    });
    let a = app.clone();
    st.on_zoom(move |delta| {
        {
            let mut e = a.tools.borrow_mut();
            if !e.genere {
                return;
            }
            // delta > 0 : on zoome (échelle plus fine)
            let max = seedmap::NIVEAUX.len() as i32 - 1;
            let niveau = (e.niveau as i32 - delta.signum()).clamp(0, max);
            if niveau as usize == e.niveau {
                return;
            }
            e.niveau = niveau as usize;
            s(&a).set_zoom_scale(facteur_zoom(&e));
        }
        redessiner(&a);
    });
    let a = app.clone();
    st.on_hover(move |fx, fy| {
        let texte = {
            let e = a.tools.borrow();
            e.rendu.as_ref().and_then(|r| {
                let (x, y) = (fx * r.image.width() as f32, fy * r.image.height() as f32);
                r.sous_pixel(x, y).map(|(bx, bz, nom)| format!("X {bx}   Z {bz}   ·   {nom}"))
            })
        };
        if let Some(t) = texte {
            s(&a).set_hover_text(t.into());
        }
    });
    let a = app.clone();
    st.on_goto_spawn(move || {
        {
            let mut e = a.tools.borrow_mut();
            if !e.genere || e.dim != 0 {
                return;
            }
            let (x, z) = seedmap::position_spawn(e.mc, e.seed);
            e.cx = x as f64;
            e.cz = z as f64;
        }
        redessiner(&a);
    });
    let a = app.clone();
    st.on_goto_origin(move || {
        {
            let mut e = a.tools.borrow_mut();
            e.cx = 0.0;
            e.cz = 0.0;
        }
        redessiner(&a);
    });
}

/// Textes qui dépendent de la langue (versions, dimensions) : rappelé au changement de langue.
pub fn maj_textes(app: &Rc<App>) {
    let st = s(app);
    let versions: Vec<String> = seedmap::VERSIONS.iter().map(|v| v.0.to_string()).collect();
    st.set_versions(modele_textes(&versions));
    let dims: Vec<String> = seedmap::DIMENSIONS.iter().map(|d| loc::t(d.0)).collect();
    st.set_dimensions(modele_textes(&dims));
    maj_pastilles(app);
}

pub fn langue_changee(app: &Rc<App>) {
    maj_textes(app);
}

fn demande(e: &Etat) -> Demande {
    Demande {
        mc: e.mc,
        dim: e.dim,
        seed: e.seed,
        cx: e.cx,
        cz: e.cz,
        niveau: e.niveau,
        genres: e.genres.iter().enumerate().filter(|(_, &on)| on).map(|(i, _)| i).collect(),
        largeur: LARGEUR,
        hauteur: HAUTEUR,
        spawn: true,
    }
}

/// Pastilles de structures de la dimension affichée.
fn maj_pastilles(app: &Rc<App>) {
    let pastilles: Vec<ToolChip> = {
        let mut e = app.tools.borrow_mut();
        e.pastilles = seedmap::GENRES.iter().enumerate().filter(|(_, g)| g.dimension == e.dim).map(|(i, _)| i).collect();
        e.pastilles
            .iter()
            .map(|&i| {
                let g = &seedmap::GENRES[i];
                ToolChip {
                    name: loc::t(&format!("Tools_S_{}", g.cle)).into(),
                    color: slint::Color::from_rgb_u8(g.couleur[0], g.couleur[1], g.couleur[2]),
                    on: e.genres[i],
                }
            })
            .collect()
    };
    s(app).set_chips(modele(pastilles));
}

/// Lit la seed saisie (ou en tire une), recentre la carte et la calcule.
fn generer(app: &Rc<App>, aleatoire: bool) {
    let st = s(app);
    let saisie = if aleatoire { String::new() } else { st.get_seed_text().to_string() };
    let seed = seedmap::lire_seed(&saisie);
    if aleatoire || saisie.trim().is_empty() {
        st.set_seed_text(seed.to_string().into());
    }
    {
        let mut e = app.tools.borrow_mut();
        e.seed = seed;
        e.cx = 0.0;
        e.cz = 0.0;
        e.niveau = NIVEAU_DEPART;
        e.genere = true;
    }
    redessiner(app);
}

/// Recalcule la carte en arrière-plan. Une seule fois à la fois : si on bouge encore pendant le calcul,
/// la dernière position demandée est calculée juste après.
fn redessiner(app: &Rc<App>) {
    {
        let mut e = app.tools.borrow_mut();
        if !e.genere {
            return;
        }
        if e.en_cours {
            e.a_refaire = true;
            return;
        }
        e.en_cours = true;
    }
    s(app).set_busy(true);
    let a = app.clone();
    lancer(async move {
        loop {
            let (d, niveau) = {
                let mut e = a.tools.borrow_mut();
                e.a_refaire = false;
                (demande(&e), e.niveau)
            };
            let rendu = a.bloquant(move || seedmap::rendre(&d)).await;
            let encore = {
                let mut e = a.tools.borrow_mut();
                if let Some(r) = rendu {
                    let image = Image::from_rgba8(images::tampon(&r.image));
                    let st = s(&a);
                    st.set_map(image);
                    st.set_has_map(true);
                    e.niveau_rendu = niveau;
                    st.set_zoom_scale(facteur_zoom(&e));
                    st.set_map_id(st.get_map_id().wrapping_add(1));
                    st.set_center_text(format!("X {}   Z {}", e.cx.round() as i64, e.cz.round() as i64).into());
                    st.set_zoom_text(format!("1:{}", seedmap::NIVEAUX[niveau].0).into());
                    e.rendu = Some(r);
                }
                if !e.a_refaire {
                    e.en_cours = false;
                }
                e.a_refaire
            };
            if !encore {
                break;
            }
        }
        s(&a).set_busy(false);
    });
}

/// Agrandissement à appliquer à la carte affichée pour qu'elle corresponde au zoom demandé.
fn facteur_zoom(e: &Etat) -> f32 {
    let bpp = |n: usize| seedmap::NIVEAUX[n].0 as f32 / seedmap::NIVEAUX[n].1 as f32;
    bpp(e.niveau_rendu) / bpp(e.niveau)
}
