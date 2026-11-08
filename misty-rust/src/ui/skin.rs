//! Onglet « Skin » de l'accueil : aperçu du personnage, changement de skin, choix de la cape.

use super::{dialogs, images, lancer, modele, App};
use crate::services::skins::{self, Profil};
use crate::services::{account, loc};
use crate::{CapeItem, HomeState, SkinState};
use slint::{ComponentHandle, Image, Model};
use std::rc::Rc;
use std::sync::atomic::{AtomicU64, Ordering};

/// Incrémenté à chaque changement de session : les chargements en cours deviennent obsolètes.
static EPOQUE: AtomicU64 = AtomicU64::new(0);

/// UUID du compte dont le skin est chargé (ou en cours de chargement).
static DERNIER_COMPTE: std::sync::Mutex<String> = std::sync::Mutex::new(String::new());

/// Nombre de capes par ligne (doit correspondre à la largeur du panneau).
const COLONNES: usize = 4;


#[derive(Default)]
pub struct Etat {
    profil: Option<Profil>,
    /// PNG du skin actuellement porté par le compte
    skin_actuel: Vec<u8>,
    /// Image de chaque cape de `profil.capes`
    capes_img: Vec<Option<Vec<u8>>>,
    /// PNG choisi par l'utilisateur, pas encore envoyé
    fichier: Option<Vec<u8>>,
    /// Incremented at each 3D preview request, to drop outdated results
    generation: u64,
    /// Chargement du profil déjà lancé (évite de le faire deux fois)
    charge: bool,
    /// Cape confirmée par Mojang (id vide = aucune) et cape demandée par l'utilisateur
    cape_confirmee: String,
    cape_voulue: String,
    envoi_cape: bool,
    /// La limite de requêtes de Mojang est atteinte (remis à zéro dès qu'une réponse arrive)
    limite_signalee: bool,
    /// Le message d'information n'a pas encore été montré à l'utilisateur
    limite_a_montrer: bool,
    /// Le message de limite concerne l'envoi d'une cape (et non le chargement du profil)
    limite_cape: bool,
}

fn s(app: &App) -> SkinState<'_> {
    app.ui.global::<SkinState>()
}

pub fn installer(app: &Rc<App>) {
    let st = s(app);

    let a = app.clone();
    st.on_open(move || {
        // Premier affichage (ou compte changé) : on va chercher le skin et les capes
        if account::session().is_some() && a.skin.borrow().profil.is_none() {
            charger(&a); // no-op si le chargement est déjà en cours (préchargé)
        } else if account::session().is_none() {
            afficher_steve(&a, false);
        }
        montrer_limite(&a); // limite atteinte pendant le préchargement : on prévient maintenant
    });

    let a = app.clone();
    st.on_pick_file(move || {
        let a = a.clone();
        lancer(async move { choisir_fichier(&a).await });
    });

    let a = app.clone();
    st.on_slim_changed(move || afficher_personnage(&a));
    let a = app.clone();
    st.on_elytra_toggled(move || afficher_personnage(&a));

    let a = app.clone();
    st.on_walk_toggled(move || {
        if s(&a).get_walk() && s(&a).get_walk_frames().row_count() == 0 {
            rendre_marche(&a);
        }
    });

    let a = app.clone();
    st.on_apply(move || {
        let a = a.clone();
        lancer(async move { appliquer(&a).await });
    });

    let a = app.clone();
    st.on_select_cape(move |id| {
        let a = a.clone();
        let id = id.to_string();
        lancer(async move { changer_cape(&a, id).await });
    });
}

/// Connexion / déconnexion : on repart de zéro.
pub fn session_changee(app: &Rc<App>) {
    // Même compte et déjà chargé (ou en cours) : on ne redemande pas tout à Mojang
    let uuid = account::session().map(|s| s.uuid).unwrap_or_default();
    {
        let mut dernier = DERNIER_COMPTE.lock().unwrap();
        if !uuid.is_empty() && *dernier == uuid && app.skin.borrow().charge {
            return;
        }
        *dernier = uuid;
    }
    EPOQUE.fetch_add(1, Ordering::SeqCst);
    *app.skin.borrow_mut() = Etat::default();
    let st = s(app);
    st.set_has_figure(false);
    st.set_frames(modele(Vec::new()));
    st.set_capes(modele(Vec::new()));
    st.set_has_file(false);
    st.set_file_name("".into());
    st.set_info("".into());
    st.set_loading(false);
    if account::session().is_some() {
        charger(app); // préchargé : l'onglet Skin s'ouvre déjà prêt
    } else if account::session().is_none() {
        afficher_steve(app, false);
    }
}

/// Pas de compte connecté : on montre Steve en entier (skin par défaut de Minecraft).
/// orce : même avec un compte connecté (secours quand rien d'autre n'est disponible).
fn afficher_steve(app: &Rc<App>, force: bool) {
    let generation = {
        let mut e = app.skin.borrow_mut();
        e.generation += 1;
        e.generation
    };
    let a = app.clone();
    lancer(async move {
        let vues = a.bloquant(|| skins::steve_tour().cloned()).await;
        if a.skin.borrow().generation != generation || (!force && account::session().is_some()) {
            return;
        }
        let st = s(&a);
        st.set_has_figure(vues.is_some());
        let images: Vec<Image> = vues.unwrap_or_default().iter().map(|v| Image::from_rgba8(images::tampon(v))).collect();
        st.set_frames(modele(images));
    });
}

pub fn langue_changee(app: &Rc<App>) {
    afficher_capes(app);
}

// ───────────────────────── Chargement ─────────────────────────

/// Ce qu'il faut pour afficher le personnage : le profil, le skin et la cape portée.
struct Charge {
    profil: Profil,
    skin: Vec<u8>,
    cape_active: Option<(usize, Vec<u8>)>,
}

/// Skin et cape portée d'un profil connu (relus sur le disque si on les a déjà eus).
async fn rassembler(profil: Profil) -> Charge {
    let skin = async {
        if profil.skin_url.is_empty() {
            Vec::new()
        } else {
            skins::telecharger(&profil.skin_url).await.unwrap_or_default()
        }
    };
    let cape = async {
        let i = profil.capes.iter().position(|c| c.active)?;
        skins::telecharger(&profil.capes[i].url).await.ok().map(|o| (i, o))
    };
    let (skin, cape_active) = futures::join!(skin, cape);
    Charge { profil, skin, cape_active }
}

/// Profil actuel demandé à Mojang.
async fn recuperer() -> anyhow::Result<Charge> {
    Ok(rassembler(skins::profil().await?).await)
}

/// Les autres capes (vignettes), sans bloquer l'affichage du personnage.
async fn recuperer_autres_capes(profil: Profil) -> Vec<Option<Vec<u8>>> {
    let telechargements = profil.capes.iter().map(|c| async move {
        if c.active {
            None
        } else {
            skins::telecharger(&c.url).await.ok()
        }
    });
    futures::future::join_all(telechargements).await
}

/// Affiche un profil (dernier connu ou réponse de Mojang). Renvoie false si le compte a changé entre-temps.
async fn afficher_charge(app: &Rc<App>, c: Charge, epoque: u64) -> bool {
    if EPOQUE.load(Ordering::SeqCst) != epoque {
        return false;
    }
    // Une cape vient d'être choisie et part chez Mojang : on ne l'écrase pas
    if app.skin.borrow().envoi_cape {
        return true;
    }
    let st = s(app);
    let profil = c.profil.clone();
    if let Some(tete) = skins::rendre_tete(&c.skin) {
        let home = app.ui.global::<HomeState>();
        home.set_has_avatar(true);
        home.set_avatar(Image::from_rgba8(images::tampon(&tete)));
    }
    {
        let mut e = app.skin.borrow_mut();
        if e.fichier.is_none() {
            st.set_slim(c.profil.fin);
        }
        e.skin_actuel = c.skin;
        e.capes_img = vec![None; profil.capes.len()];
        if let Some((i, octets)) = c.cape_active {
            e.capes_img[i] = Some(octets);
        }
        e.cape_confirmee = profil.capes.iter().find(|x| x.active).map(|x| x.id.clone()).unwrap_or_default();
        e.cape_voulue = e.cape_confirmee.clone();
        e.profil = Some(c.profil);
    }
    afficher_personnage(app);
    afficher_capes(app);
    st.set_loading(false);

    // Les vignettes des autres capes arrivent après
    let autres = app.bg(recuperer_autres_capes(profil)).await;
    if EPOQUE.load(Ordering::SeqCst) != epoque {
        return false;
    }
    {
        let mut e = app.skin.borrow_mut();
        for (i, octets) in autres.into_iter().enumerate() {
            if octets.is_some() && i < e.capes_img.len() {
                e.capes_img[i] = octets;
            }
        }
    }
    afficher_capes(app);
    true
}

/// Mojang limite les demandes : on le dit à l'utilisateur (une fois par épisode).
fn signaler_limite(app: &Rc<App>) {
    {
        let mut e = app.skin.borrow_mut();
        if e.limite_signalee {
            return;
        }
        e.limite_signalee = true;
        e.limite_a_montrer = true;
    }
    montrer_limite(app);
}

/// Montre le message si l'utilisateur regarde l'onglet Skin (sinon il attend qu'il l'ouvre).
fn montrer_limite(app: &Rc<App>) {
    let regarde = s(app).get_tab() == 1 && app.ui.global::<crate::AppState>().get_page() == 0;
    if !regarde || !std::mem::take(&mut app.skin.borrow_mut().limite_a_montrer) {
        return;
    }
    let a = app.clone();
    let texte = if std::mem::take(&mut app.skin.borrow_mut().limite_cape) { loc::t("Skin_LimiteCape_Popup") } else { loc::t("Skin_Limite_Popup") };
    lancer(async move { dialogs::info(&a, &loc::t("Skin_Limite_Titre"), &texte).await });
}

/// Charge le skin et les capes : d'abord le dernier profil connu (disque), puis Mojang.
/// Si Mojang refuse (limite de requêtes) ou ne répond pas, on garde l'affichage et on réessaie
/// tout seul, en patientant de plus en plus longtemps, jusqu'à confirmer la configuration.
fn charger(app: &Rc<App>) {
    {
        let mut e = app.skin.borrow_mut();
        if e.charge {
            return;
        }
        e.charge = true;
    }
    let epoque = EPOQUE.load(Ordering::SeqCst);
    let a = app.clone();
    lancer(async move {
        let st = s(&a);
        st.set_loading(true);
        st.set_info("".into());
        let uuid = account::session().map(|s| s.uuid).unwrap_or_default();

        // 1. Dernier skin connu : affiché tout de suite, sans aucune demande à Mojang
        let mut affiche = false;
        if let Some(p) = skins::profil_en_cache(&uuid) {
            let c = a.bg(rassembler(p)).await;
            affiche = afficher_charge(&a, c, epoque).await;
            if EPOQUE.load(Ordering::SeqCst) != epoque {
                return;
            }
        }

        // 2. Profil réel chez Mojang
        let mut essai = 0usize;
        loop {
            let resultat = a.bg(recuperer()).await;
            if EPOQUE.load(Ordering::SeqCst) != epoque {
                return; // le compte a changé entre-temps
            }
            match resultat {
                Ok(c) => {
                    skins::enregistrer_profil(&uuid, &c.profil);
                    {
                        let mut e = a.skin.borrow_mut();
                        e.limite_signalee = false;
                        e.limite_a_montrer = false;
                    }
                    st.set_info("".into());
                    afficher_charge(&a, c, epoque).await; // confirme (ou corrige) ce qui était affiché
                    return;
                }
                Err(e) => {
                    let limite = e.downcast_ref::<skins::Limite>().map(|l| l.attente);
                    if limite.is_some() {
                        signaler_limite(&a);
                    }
                    let message = if limite.is_some() { loc::t("Skin_Limite") } else { loc::tf("Skin_ErreurChargement", &[&e]) };
                    st.set_info(message.into());
                    if !affiche {
                        st.set_loading(false);
                        afficher_steve(&a, true); // rien en mémoire : Steve en attendant
                    }
                    essai += 1;
                    if essai > 8 {
                        a.skin.borrow_mut().charge = false; // on réessaiera à la prochaine ouverture
                        return;
                    }
                    let attente = limite.filter(|&n| n > 0).unwrap_or(match essai {
                        1 => 5,
                        2 => 10,
                        3 => 20,
                        4 => 40,
                        _ => 60,
                    });
                    let attente = attente.clamp(2, 120);
                    a.bg(async move { tokio::time::sleep(std::time::Duration::from_secs(attente)).await }).await;
                    if EPOQUE.load(Ordering::SeqCst) != epoque {
                        return;
                    }
                }
            }
        }
    });
}
/// Preview in 3D: the chosen skin if any, otherwise the current skin, with the equipped cape.
/// Calcule les images de la marche (en arrière-plan) pour le skin et la cape affichés.
fn rendre_marche(app: &Rc<App>) {
    let fin = s(app).get_slim();
    let elytres = s(app).get_elytra();
    let (png, cape, generation) = {
        let e = app.skin.borrow();
        let mut png = e.fichier.clone().unwrap_or_else(|| e.skin_actuel.clone());
        if png.is_empty() {
            png = include_bytes!("../../assets/steve.png").to_vec();
        }
        let cape = e.profil.as_ref().and_then(|p| p.capes.iter().position(|c| c.active)).and_then(|i| e.capes_img.get(i).cloned().flatten());
        (png, cape, e.generation)
    };
    let a = app.clone();
    lancer(async move {
        let vues = a.bloquant(move || skins::rendre_marche(&png, fin, cape.as_deref(), elytres)).await;
        if a.skin.borrow().generation != generation {
            return;
        }
        if let Some(vues) = vues {
            let images: Vec<Image> = vues.iter().map(|v| Image::from_rgba8(images::tampon(v))).collect();
            s(&a).set_walk_frames(modele(images));
        }
    });
}

fn afficher_personnage(app: &Rc<App>) {
    s(app).set_walk_frames(modele(Vec::new()));
    let fin = s(app).get_slim();
    let elytres = s(app).get_elytra();
    let (png, cape, generation) = {
        let mut e = app.skin.borrow_mut();
        e.generation += 1;
        let png = e.fichier.clone().unwrap_or_else(|| e.skin_actuel.clone());
        let cape = e.profil.as_ref().and_then(|p| p.capes.iter().position(|c| c.active)).and_then(|i| e.capes_img.get(i).cloned().flatten());
        (png, cape, e.generation)
    };
    let a = app.clone();
    lancer(async move {
        // 1. La face seule, affichée tout de suite (le personnage ne tourne pas encore)
        let st = s(&a);
        // Déjà un personnage affiché (changement de cape...) : on le garde jusqu'à la fin du calcul
        if !st.get_has_figure() {
            let (png2, cape2) = (png.clone(), cape.clone());
            let face = a.bloquant(move || skins::rendre_vues(&png2, fin, cape2.as_deref(), &[0])).await;
            if a.skin.borrow().generation != generation {
                return; // a newer preview was requested meanwhile
            }
            match face.and_then(|mut v| v.pop()) {
                Some(v) => {
                    let image = Image::from_rgba8(images::tampon(&v));
                    st.set_has_figure(true);
                    st.set_frames(modele(vec![image; skins::IMAGES_TOUR]));
                }
                None => {
                    st.set_has_figure(false);
                    return;
                }
            }
        }
        // 2. Le tour complet (calculé sur tous les cœurs)
        let vues = a.bloquant(move || skins::rendre_tour_elytres(&png, fin, cape.as_deref(), elytres)).await;
        if a.skin.borrow().generation != generation {
            return;
        }
        if let Some(vues) = vues {
            let images: Vec<Image> = vues.iter().map(|v| Image::from_rgba8(images::tampon(v))).collect();
            st.set_frames(modele(images));
            if st.get_walk() {
                rendre_marche(&a);
            }
        }
    });
}

fn afficher_capes(app: &Rc<App>) {
    let e = app.skin.borrow();
    let Some(profil) = &e.profil else { return };
    let aucune_active = !profil.capes.iter().any(|c| c.active);
    let mut elements = vec![CapeItem {
        id: "".into(),
        name: loc::t("Skin_AucuneCape").into(),
        image: Image::default(),
        has_image: false,
        active: aucune_active,
        col: 0,
        row: 0,
    }];
    for (i, cape) in profil.capes.iter().enumerate() {
        let image = e.capes_img.get(i).cloned().flatten().and_then(|o| skins::rendre_cape(&o)).map(|i| Image::from_rgba8(images::tampon(&i)));
        elements.push(CapeItem {
            id: cape.id.clone().into(),
            name: cape.nom.clone().into(),
            has_image: image.is_some(),
            image: image.unwrap_or_default(),
            active: cape.active,
            col: 0,
            row: 0,
        });
    }
    for (i, el) in elements.iter_mut().enumerate() {
        el.col = (i % COLONNES) as i32;
        el.row = (i / COLONNES) as i32;
    }
    let lignes = elements.len().div_ceil(COLONNES).max(1) as i32;
    drop(e);
    let st = s(app);
    st.set_cape_rows(lignes);
    st.set_capes(modele(elements));
}

// ───────────────────────── Actions ─────────────────────────

async fn choisir_fichier(app: &Rc<App>) {
    let Some(chemin) = dialogs::choisir_fichier(&loc::t("Skin_Fichier"), "PNG", &["png"]).await else { return };
    let octets = match std::fs::read(&chemin) {
        Ok(o) => o,
        Err(e) => return dialogs::erreur(app, &loc::t("Skin_Erreur"), &e.to_string()).await,
    };
    if !skins::skin_valide(&octets) {
        return dialogs::erreur(app, &loc::t("Skin_Erreur"), &loc::t("Skin_Invalide")).await;
    }
    app.skin.borrow_mut().fichier = Some(octets);
    let st = s(app);
    st.set_has_file(true);
    st.set_file_name(chemin.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default().into());
    st.set_info("".into());
    afficher_personnage(app);
}

async fn appliquer(app: &Rc<App>) {
    let Some(png) = app.skin.borrow().fichier.clone() else { return };
    let st = s(app);
    let fin = st.get_slim();
    st.set_busy(true);
    st.set_info("".into());
    let envoi = png.clone();
    match app.bg(async move { skins::changer_skin(envoi, fin).await }).await {
        Ok(()) => {
            {
                let mut e = app.skin.borrow_mut();
                e.skin_actuel = png.clone();
                e.fichier = None;
                if let Some(p) = e.profil.as_mut() {
                    p.fin = fin;
                }
            }
            st.set_has_file(false);
            st.set_file_name("".into());
            if let Some(s) = account::session() {
                skins::memoriser_skin_envoye(&s.uuid, &png, fin);
            }
            st.set_info(loc::t("Skin_Applique").into());
            afficher_personnage(app);
            // La pastille de l'accueil montre la nouvelle tête tout de suite
            if let Some(tete) = skins::rendre_tete(&png) {
                let home = app.ui.global::<HomeState>();
                home.set_has_avatar(true);
                home.set_avatar(Image::from_rgba8(images::tampon(&tete)));
            }
        }
        Err(e) => {
            let message = if e.downcast_ref::<skins::Limite>().is_some() { loc::t("Skin_LimiteEchec") } else { e.to_string() };
            dialogs::erreur(app, &loc::t("Skin_Erreur"), &message).await
        }
    }
    st.set_busy(false);
}

/// Applique une cape dans l'interface (sans attendre le serveur).
fn cape_localement(app: &Rc<App>, id: &str) {
    if let Some(p) = app.skin.borrow_mut().profil.as_mut() {
        for c in p.capes.iter_mut() {
            c.active = c.id == id;
        }
    }
    afficher_capes(app);
    afficher_personnage(app);
}

/// La cape change tout de suite à l'écran ; la demande part ensuite à Mojang.
/// Si l'utilisateur change encore d'avis pendant l'envoi, seule la dernière demande compte.
async fn changer_cape(app: &Rc<App>, id: String) {
    let st = s(app);
    {
        let mut e = app.skin.borrow_mut();
        if e.cape_voulue == id {
            return;
        }
        e.cape_voulue = id.clone();
    }
    st.set_info("".into());
    cape_localement(app, &id);

    if std::mem::replace(&mut app.skin.borrow_mut().envoi_cape, true) {
        return; // un envoi est déjà en cours : il enverra aussi cette dernière demande
    }
    let mut essai = 0usize;
    loop {
        // Courte pause : si l'utilisateur clique encore sur d'autres capes, une seule demande partira
        app.bg(async { tokio::time::sleep(std::time::Duration::from_millis(500)).await }).await;
        let voulue = app.skin.borrow().cape_voulue.clone();
        if voulue == app.skin.borrow().cape_confirmee {
            app.skin.borrow_mut().envoi_cape = false;
            return; // revenu à la cape déjà portée : rien à envoyer
        }
        let envoi = voulue.clone();
        let resultat = app
            .bg(async move { skins::activer_cape(if envoi.is_empty() { None } else { Some(envoi.as_str()) }).await })
            .await;
        match resultat {
            Ok(()) => {
                let mut e = app.skin.borrow_mut();
                e.cape_confirmee = voulue.clone();
                if let (Some(p), Some(s)) = (e.profil.as_ref(), account::session()) {
                    skins::enregistrer_profil(&s.uuid, p); // dernier état connu
                }
                if e.cape_voulue == voulue {
                    e.envoi_cape = false;
                    e.limite_signalee = false;
                    drop(e);
                    st.set_info(loc::t("Skin_CapeChangee").into());
                    return;
                }
                // une autre cape a été demandée entre-temps : on l'envoie
            }
            Err(e) => {
                // Mojang limite les demandes : la cape reste affichée, on réessaie en patientant
                if let Some(limite) = e.downcast_ref::<skins::Limite>() {
                    essai += 1;
                    if essai <= 4 {
                        app.skin.borrow_mut().limite_cape = true;
                        signaler_limite(app);
                        let attente = (if limite.attente > 0 { limite.attente } else { 5u64 << (essai - 1).min(3) }).clamp(2, 120);
                        app.bg(async move { tokio::time::sleep(std::time::Duration::from_secs(attente)).await }).await;
                        continue;
                    }
                }
                let (confirmee, message) = {
                    let mut etat = app.skin.borrow_mut();
                    etat.envoi_cape = false;
                    etat.cape_voulue = etat.cape_confirmee.clone();
                    let message = if e.downcast_ref::<skins::Limite>().is_some() { loc::t("Skin_LimiteEchec") } else { e.to_string() };
                    (etat.cape_confirmee.clone(), message)
                };
                cape_localement(app, &confirmee); // on revient à la cape réellement portée
                dialogs::erreur(app, &loc::t("Skin_ErreurCape"), &message).await;
                return;
            }
        }
    }
}
