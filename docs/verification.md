# Preverjanje faze 01

Windows: Rust 1.100.0 MSVC, Node 24.15.0, Visual Studio C++ in WebView2.

Uspešno: npm install (0 ranljivosti), TS/ESLint, obe Vite gradnji, cargo check/build/test, Clippy. Ogrodje še nima vsebinskih unit testov (cargo test: 0 testov). CLI --help je izveden; dejanski GET /health vrne ok. Dashboard strežnik vrne HTTP 200. Tauri dev odpre okno; vidni Preflight 0.1.0 potrdi klic get_app_info.

TypeScript 6 je izbran zaradi podprtega razpona typescript-eslint. V tej seji je bilo treba dodati cargo bin na PATH in namestiti rustup komponento clippy. CI vsebuje Windows/macOS/Linux; rezultati oddaljenega CI se preverjajo po pushu.

Faze se po izrecnem uporabnikovem naročilu razvijajo zaporedno do 03, na ločenih vejah. PR-ji niso samodejno združeni.

# Preverjanje faze 02

26 Rust testov uspešnih (5 CLI, 6 unit, 15 engine). Vključeni so simulirani manjkajoči programi, Python launcher, Java/JDK, neveljavni profili, zasebnost in resnična testa timeouta/omejitve izhoda s testnim podprocesom. Testi ne potrebujejo lokalne Jave. Cargo fmt, Clippy in lokalna gradnja so preverjeni.

Pravi profil python-workshop.yaml na razvojnem Windows računalniku vrne dva pass in eno neobvezno opozorilo (Node je zunaj zahtevanega razpona 22); text in JSON vrneta exit 0. Izpis ne vsebuje poti ali surovega stdout/stderr. Podrobne lokalne rezultate namenoma hranimo zunaj Gita.
