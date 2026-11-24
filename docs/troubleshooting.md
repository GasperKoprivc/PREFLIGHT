# Odpravljanje težav

V namiznem pregledu razširi vrstico preverjanja. Vodič upošteva razlog (`reason_code`) in operacijski sistem; sistem lahko tudi ročno izbereš. Po popravku klikni **Ponovno preveri to zahtevo**. Ponovi se samo izbrano dovoljeno preverjanje, druge vrstice ostanejo iz zadnjega pregleda. Če želiš sveže rezultate celotnega okolja, uporabi gumb na vrhu. Rezultati se učitelju ne posodobijo samodejno.

## Python ni dosegljiv

Windows: uporabi [uradni Python install manager in navodila za odpravljanje težav](https://docs.python.org/3/using/windows.html). Preveri `python --version` in `py -3 --version`; če ukaz odpre Store ali drugo namestitev, preveri vzdevke za izvajanje aplikacij in PATH. macOS: uporabi ustrezni paket s [python.org](https://www.python.org/downloads/) in `python3 --version`. Linux: uporabi repozitorij svoje distribucije in `python3`, ne spreminjaj sistemskega Pythona. Izberi različico, ki ustreza delavnici.

## Napačna različica ali PATH

PATH določa, katero namestitev sistem najprej najde. Windows `where.exe python` / `where.exe node` / `where.exe git` / `where.exe java`; macOS/Linux `command -v python3` / `command -v node` / `command -v git` / `command -v java`. Preveri vrstni red map in obstoječi upravljalnik različic. Po namestitvi ali spremembi PATH zapri in ponovno odpri terminal ter Preflight; če aplikacija podeduje staro okolje, je lahko potrebna ponovna prijava uporabnika.

Node: [uradni prenosi](https://nodejs.org/en/download) omogočajo izbiro različice. Najnovejši LTS ni nujno pravilen za profil, ki zahteva Node 22 in <23. Git: uporabi [uradne možnosti za svoj OS](https://git-scm.com/install/). Na Windows je v vodiču tudi ročni ukaz winget za Git, če je winget že na voljo; paket lahko namesti novejšo različico, zato po namestitvi preveri zahtevo profila.

## Java deluje, JDK pa ne

Potrebna sta `java -version` in `javac -version`. Namesti celoten JDK za zahtevano glavno različico z [uradne strani ponudnika](https://learn.microsoft.com/en-us/java/openjdk/download). JAVA_HOME kaže na koren JDK, PATH mora vključevati njegov bin. Java in javac morata kazati na isto glavno različico. Na macOS `/usr/libexec/java_home -V` našteje nameščene JDK-je; na Linux uporabi orodja svoje distribucije za izbiro JDK. Ne uporabljaj postopkov druge distribucije brez preverjanja.

## VS Code ali razširitev

`vscode_unavailable` pomeni, da CLI ni dosegljiv, ne da razširitev zagotovo manjka. V Extensions ročno preveri točen ID. Na macOS uporabi ukazno paleto za **Shell Command: Install code command in PATH**; na Windows preveri namestitveno možnost PATH, na Linux uradno namestitev in dosegljivost `code`. Diagnostika uporablja privzeti profil; druge profile ali prenosne namestitve preveri ročno.

[Uradni CLI VS Code](https://code.visualstudio.com/docs/configure/command-line) podpira seznam in namestitev razširitev. Kopirni namestitveni ukazi so v kodi dovoljeni samo za tri vgrajene ID-je (`ms-python.python`, `redhat.java`, `dbaeumer.vscode-eslint`); poljuben ID iz YAML nikoli ne postane ukaz. Druge razširitve poišči ročno in preveri izdajatelja. Namestitev lahko pridobi dodatne odvisnosti.

## Timeout, disk in HTTPS

Timeout, neprepoznana različica in prevelik izhod pomenijo neznan rezultat. Preveri odzivnost programa v terminalu in ponovi; nikoli jih ne obravnavaj kot uspeh. Za disk preveri prostor na disku delovne mape in odstranjuj samo svoje nepotrebne datoteke. Za HTTPS preveri izrecni vklop, internet, DNS in proxy; Preflight diagnostičnih testov ne izvaja prek sistemskega proxyja. Blokiranih zasebnih naslovov ali TLS zaščite ne obidi. HEAD lahko strežnik zavrne, zato to ni dokaz uspešne dosegljivosti.

V tej fazi Preflight ničesar ne namešča. Vsi ukazi so samo za kopiranje in ročno uporabo. Za neznan sistem ali nestandardno namestitev je prikazan ročni fallback; navodila za macOS/Linux niso bila fizično izvedena v tej Windows seji.
