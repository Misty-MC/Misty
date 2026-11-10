// build.rs — compile l'interface Slint (../ui/main.slint et tous les fichiers qu'il importe).

fn main() {
    // Curseurs personnalisés (main du studio de skin) : encore une fonction expérimentale de Slint 1.18
    std::env::set_var("SLINT_ENABLE_EXPERIMENTAL_FEATURES", "1");
    slint_build::compile("../ui/main.slint").expect("compilation de l'interface Slint");
}
