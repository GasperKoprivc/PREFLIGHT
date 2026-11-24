import type { Check, CheckResult } from "./model";
export type OperatingSystem = "windows" | "macos" | "linux" | "other";
export interface Guide {
  title: string;
  steps: string[];
  links: { label: string; url: string }[];
  commands: { label: string; command: string }[];
}
const pages = {
  python: "https://www.python.org/downloads/",
  git: "https://git-scm.com/install/",
  node: "https://nodejs.org/en/download",
  java: "https://learn.microsoft.com/en-us/java/openjdk/download",
  vscode: "https://code.visualstudio.com/docs/configure/command-line",
};
const extensions: Record<string, string> = {
  "ms-python.python": "code --install-extension ms-python.python",
  "redhat.java": "code --install-extension redhat.java",
  "dbaeumer.vscode-eslint": "code --install-extension dbaeumer.vscode-eslint",
};
export function guidance(
  check: Check,
  result: CheckResult,
  os: OperatingSystem,
): Guide {
  const guide: Guide = {
    title: "Kako naprej",
    steps: [],
    links: [],
    commands: [],
  };
  const reason = result.reason_code;
  if (os === "other") {
    guide.steps = [
      "Za ta sistem ni preverjenega postopka. Izberi Windows, macOS ali Linux, če ustreza tvojemu okolju, sicer prosi izvajalca delavnice za pomoč.",
    ];
    return guide;
  }
  if (
    reason === "satisfied" ||
    ["extension_present", "disk_available", "https_reachable"].includes(reason)
  ) {
    guide.title = "Preverjanje je uspešno";
    guide.steps = [
      "Sprememba ni potrebna. Po spremembi okolja lahko preverjanje ponoviš.",
    ];
    return guide;
  }
  if (
    reason === "timeout" ||
    reason === "output_limit" ||
    reason === "process_failed" ||
    reason === "unrecognized_version"
  ) {
    guide.title = "Rezultata ni mogoče potrditi";
    guide.steps = [
      "To ne pomeni nujno, da orodje manjka. Preveri, ali se ukaz v novem terminalu odzove in vrne stabilno različico.",
      "Zapri zataknjene pozive orodja, preveri dovoljenja in ponovno zaženi Preflight. Surovih izpisov ne pošiljaj učitelju.",
    ];
  }
  if (check.type === "runtime" && check.tool) {
    const tool = check.tool;
    if (!["python", "git", "node", "java"].includes(tool)) {
      guide.steps = ["Neznano orodje. Preveri profil."];
      return guide;
    }
    guide.links.push({ label: "Uradna namestitev", url: pages[tool] });
    if (reason === "version_mismatch") {
      guide.title = "Izberi zahtevano različico";
      guide.steps.push(
        "Nameščena različica je prepoznana, vendar ne ustreza omejitvi profila. Na uradni strani izberi ustrezno glavno različico; najnovejša izdaja ni vedno prava.",
        "Če imaš več namestitev, preveri vrstni red PATH. Uporabi tisto, ki ustreza zahtevi, prikazani nad tem vodičem.",
      );
    }
    if (reason === "not_found") {
      guide.title = "Orodje manjka ali ni na PATH";
      guide.steps.push(
        "Če je orodje že nameščeno, preveri njegov imenik bin na PATH. PATH je seznam map, v katerih sistem išče ukaze. Sicer uporabi uradno namestitev za svoj sistem in zahtevano različico.",
      );
    }
    const verify =
      tool === "python"
        ? os === "windows"
          ? "py -3 --version"
          : "python3 --version"
        : tool === "git"
          ? "git --version"
          : tool === "node"
            ? "node --version"
            : "java -version";
    guide.commands.push({ label: "Preveri različico", command: verify });
    if (tool === "python") {
      if (os === "windows") {
        guide.steps.push(
          "Na Windows namesti Python z uradne strani (Python install manager). Če ukaz python odpre Store ali drugo različico, preveri upravljalnik Python in vzdevke za izvajanje aplikacij; Preflight poskusi tudi py -3.",
        );
        guide.links.push({
          label: "Python na Windows in odpravljanje težav",
          url: "https://docs.python.org/3/using/windows.html",
        });
      } else if (os === "macos")
        guide.steps.push(
          "Na macOS izberi uradni namestitveni paket za svoj procesor. Uporabljaj python3; ne spreminjaj ali odstranjuj sistemskega Pythona.",
        );
      else
        guide.steps.push(
          "Na Linux uporabi paket python3 iz zaupanja vrednih repozitorijev distribucije oziroma navodila python.org. Ne spreminjaj povezave sistemskega python in ne nameščaj paketov prek sudo pip.",
        );
    }
    if (tool === "node")
      guide.steps.push(
        "Na strani Node.js izberi različico iz profila (npr. 22, če profil zahteva <23). Namestitveni paket izberi za svoj OS in procesor; obstoječi upravljalnik različic lahko kaže na drugo namestitev.",
      );
    if (tool === "git") {
      guide.steps.push(
        os === "windows"
          ? "Na Windows uporabi Git for Windows; v namestitvi omogoči uporabo Git iz ukazne vrstice."
          : os === "macos"
            ? "Na macOS uporabi uradne možnosti Git; Xcode Command Line Tools lahko namestijo Apple Git, ki je lahko starejši od zahteve."
            : "Na Linux uporabi paket git iz repozitorija svoje distribucije; preveri različico paketa pred namestitvijo.",
      );
      if (os === "windows")
        guide.commands.push({
          label:
            "Ročna namestitev Git (če imaš winget; preveri zahtevano različico)",
          command: "winget install --id Git.Git --exact --source winget",
        });
    }
    if (tool === "java") {
      guide.commands.push({
        label: "Preveri prevajalnik JDK",
        command: "javac -version",
      });
      guide.title =
        reason === "jdk_missing"
          ? "Potrebuješ celoten JDK"
          : reason === "jdk_mismatch"
            ? "Java in javac se ne ujemata"
            : guide.title;
      guide.steps.push(
        "Namesti JDK, ne samo JRE, za zahtevano glavno različico. Java in javac morata kazati na isto glavno različico. JAVA_HOME kaže na koren JDK, PATH pa mora vključevati njegov bin.",
      );
      guide.steps.push(
        os === "windows"
          ? "V nastavitvah okoljskih spremenljivk preveri JAVA_HOME in vrstni red JDK map v PATH."
          : os === "macos"
            ? "Na macOS lahko /usr/libexec/java_home -V prikaže nameščene JDK-je; v svojem terminalu izberi ustrezni JAVA_HOME."
            : "Na Linux preveri izbrani java in javac prek orodij svoje distribucije (npr. alternatives), oba morata pripadati ustreznemu JDK.",
      );
    }
    guide.commands.push({
      label: "Poišči izbrano namestitev",
      command:
        os === "windows"
          ? `where.exe ${tool === "python" ? "python" : tool}`
          : `command -v ${tool === "python" ? "python3" : tool}`,
    });
    guide.steps.push(
      "Po spremembi PATH ali namestitvi zapri in znova odpri terminal ter Preflight. Nato ponovno preveri; za preverjanje novega okolja je včasih potreben odjavljen in znova prijavljen uporabnik.",
    );
  } else if (check.type === "vscode_extension") {
    guide.links.push({
      label: "Uradna navodila VS Code CLI",
      url: pages.vscode,
    });
    guide.commands.push({
      label: "Seznam razširitev privzetega profila",
      command: "code --list-extensions",
    });
    if (reason === "vscode_unavailable") {
      guide.title = "VS Code CLI ni dosegljiv";
      guide.steps.push(
        "To ni dokaz, da razširitev manjka. V VS Code ročno odpri Extensions in preveri točen ID iz profila. Preverjanje uporablja privzeti profil VS Code.",
      );
      guide.steps.push(
        os === "macos"
          ? "V ukazni paleti VS Code izberi Shell Command: Install code command in PATH, nato znova odpri terminal in aplikacijo."
          : os === "windows"
            ? "Pri namestitvi VS Code omogoči dodajanje na PATH oziroma preveri njegovo mapo bin. Prenosne in nestandardne namestitve preveri ročno."
            : "Preveri, ali je code iz uradne namestitve dosegljiv na PATH. Namestitve v izoliranih paketih lahko zahtevajo ročni pregled.",
      );
    } else {
      guide.title = "Namesti zahtevano razširitev";
      guide.steps.push(
        "V VS Code odpri Extensions, poišči točen ID, prikazan pri zahtevi, preveri izdajatelja in klikni Install. Uporabi isti profil VS Code kot pri preverjanju.",
      );
      const install = check.extension ? extensions[check.extension] : undefined;
      if (install)
        guide.commands.push({
          label:
            "Ročna namestitev znane razširitve (lahko prenese dodatne odvisnosti)",
          command: install,
        });
    }
    guide.steps.push(
      "Po namestitvi ponovno naloži okno VS Code in ponovi preverjanje. Če uporabljaš drug urejevalnik, se o zahtevi dogovori z izvajalcem delavnice.",
    );
  } else if (check.type === "disk_space") {
    guide.title =
      reason === "disk_low"
        ? "Sprosti prostor na delovnem disku"
        : "Prostora ni mogoče potrditi";
    guide.steps.push(
      "Preverja se disk trenutne delovne mape, ne nujno vseh diskov računalnika. V sistemskem upravljalniku prostora preglej zasedenost.",
      "Premakni ali odstrani samo datoteke, za katere veš, da jih ne potrebuješ, in preveri koš. Preflight sam ne briše ničesar. Nato ponovi preverjanje.",
    );
  } else if (check.type === "https") {
    guide.title = "Preveri omrežno dosegljivost";
    guide.steps.push(
      reason === "network_skipped"
        ? "Spletno preverjanje je izključeno. Če ga želiš izvesti, ga izrecno omogoči pri profilu."
        : reason === "unsafe_url"
          ? "Naslov ali preusmeritev ni varen javni HTTPS cilj. Prosi avtorja profila za javni naslov na vratih 443. Ne izklapljaj zaščite certifikatov ali preverjanja naslovov."
          : "Preveri internet, DNS, šolski proxy in dostop do navedene strani v brskalniku. Napaka HEAD lahko pomeni omejitev strežnika; ne šteje kot uspeh.",
      "Preflight ne uporablja sistemskega proxyja za diagnostične teste in ne prenaša vsebine strani. Po ureditvi povezave ponovi preverjanje.",
    );
  }
  if (!guide.steps.length)
    guide.steps.push(
      "Razlog še nima posebnega postopka. Preveri zahtevo z izvajalcem delavnice in ponovi preverjanje.",
    );
  return guide;
}
