# Omejeni samodejni popravki

Začetna matrika vsebuje samo **Windows x64 → Git.Git 2.56.1 → winget**. Paket, izdajatelj, namestitveni URL in hash so bili preverjeni z lokalnim `winget show --id Git.Git --exact --source winget --disable-interactivity`. Gre za Git for Windows, izdajatelj The Git Development Community, vir namestitvenega programa github.com/git-for-windows/git. Winget preveri hash manifesta; Preflight ne izklopi te zaščite. Če se paket umakne ali profil zahteva drugo različico, uporabi ročni vodič.

Akcija se ponudi samo za lokalno potrjeno manjkajoči Git ali neustrezno različico in le, če omejitev profila dovoljuje 2.56.1. Ne uporablja se za unknown/timeout. macOS, Linux, ARM64, Python, Node in Java v tej fazi ostanejo ročno vodeni. Brez avtomatskega apt, sudo ali prenosa zagonskih skript.

1. Izvedi pregled okolja in razširi Git vrstico.
2. Klikni **Preglej namestitev Git**. Pokažejo se program, paket, različica, upravljalnik, točen ukaz, posledice, licenca in morebitna administratorska potrditev.
3. Samo če želiš to spremembo računalnika, klikni **Potrdi namestitev**. Preklic ničesar ne namesti. Potrditev velja pet minut, je enkratna in vezana na nespremenjeni lokalni profil/pregled.
4. Med namestitvijo počakaj na status in morebitni sistemski poziv Windows, ki ga potrdiš sam. Nobena licenca ali sistemski poziv se ne potrdi samodejno.
5. Po zaključku se preverjanje Git ponovi; po spremembi PATH znova odpri Preflight in izvedi celoten pregled. Rezultati se ne delijo samodejno.

Ukaz je v kodi fiksen:

```text
winget install --id Git.Git --exact --source winget --version 2.56.1 --architecture x64 --silent --disable-interactivity
```

Winget se zaganja neposredno prek znane App Installer alias poti v lokalnem WindowsApps, ne prek lupine ali trenutne projektne mape. Vir winget in App Installer morata biti že normalno nastavljena; brez tihega sprejemanja pogojev vira/paketa. Uporabnik, ki lahko spremeni registracijo App Installerja ali njegov vir, ostaja zunaj varnostne meje te aplikacije.

Runner ima omejitev pet minut in 1 MiB izhoda. Preflight ne vrača surovih izhodov ali poti. Winget/namestitveni program lahko sam ustvari lokalne sistemske dnevnike; niso deljeni z učiteljem. Ob timeoutu ali prevelikem izhodu je izid neznan, saj namestitveni podproces lahko še deluje; nadaljnje namestitve v tej seji aplikacije so blokirane. Najprej ročno preveri stanje namestitvenega programa in šele nato ponovno zaženi Preflight. Ponovljena namestitev brez pregleda lahko povzroči sočasne namestitvene programe.

**Dejansko preverjeno:** dosegljivost upravljalnika in metapodatki paketa na Windows, testi fiksnega registra, razpona različic, potrditev/preklica in rezultatov s simuliranim runnerjem. **Še nepreverjeno:** resnična namestitev/povišanje pravic prek tega UI. Testi nikoli ne nameščajo paketov. Zato sprejemni pogoj faze 08 za ročno potrjeno namestitev ostaja odprt do uporabniškega smoke testa; uspešen CI tega ne dokazuje.

Uradna specifikacija zastavic: https://learn.microsoft.com/en-us/windows/package-manager/winget/install
