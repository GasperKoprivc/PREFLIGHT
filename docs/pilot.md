# Prvi pilot v razredu (10–15 udeležencev)

## Pred uro

1. Šola pregleda docs/privacy.md: pravno podlago, odgovorno osebo, gostovanje, hrambo in dnevnike proxyja. Aplikacija sama ni zagotovilo skladnosti z GDPR.
2. Skrbnik na ciljnih računalnikih preveri zagon nepodpisanega paketa po odobrenem postopku. SmartScreen/Gatekeeper lahko zahtevata pregled. Zaščit ne izklapljaj.
3. Preveri HTTPS, /health, trajni SQLite disk, backup/restore in čiščenje poteklih sob. Pilot uporablja eno instanco API, največ 600 zahtev/minuto.
4. Profil brez osebnih podatkov preizkusi na pripravljenem in nepopolnem okolju. Udeležencem razloži morebitne HTTPS teste.
5. Preveri izvor artefakta, commit, arhitekturo in SHA256SUMS. Učiteljsko skrivnost varno shrani; udeležencem daj samo kodo/povezavo.

## Potek ure

1. Najprej pokaži lokalni pregled brez sobe. Udeleženci lahko ostanejo pri lokalni diagnostiki.
2. Ustvari sobo. Udeleženci vnesejo API izvor in kodo, preberejo preneseni profil ter ga lokalno preverijo.
3. Pred deljenjem vsak pregleda minimalni JSON in izrecno privoli. En testni udeleženec naj ne deli; dashboard pri njem ne sme pokazati rezultatov. Obvezni unknown ni uspeh.
4. Učitelj spremlja agregate, čas osvežitve in težavne preverbe; pomoč poteka brez imen udeležencev.
5. Začni z ročnimi vodiči. Samodejni Git popravek preizkusi samo na testnem Windows x64 računalniku pod nadzorom skrbnika, po prikazu ukaza in dodatnem kliku Potrdi namestitev. Gesel ne vnašaj v klepet. Ob neznanem izidu ročno preveri namestitveni program.
6. Po popravku ponovi zahtevo ali celoten pregled. Nova posodobitev učitelju zahteva novo privolitev.

## Zaključek

- Dva udeleženca izbrišeta seji in potrdita izginotje iz pregleda. Učitelj nato izbriše sobo. Potek preveri v ločenem testnem okolju s TTL 60 sekund.
- Povratne informacije naj vsebujejo samo OS/arhitekturo, korak, razlog težave, čas do rešitve in prostovoljno mnenje. Brez imen, surovih logov, poti, strojnih ID-jev ali posnetkov žetonov.
- Vprašaj, ali je bilo jasno, kaj se deli, ali je deloval lokalni pregled, ali je bil nasvet uporaben in ali je bil izbris očiten.

## Evidenca pred oznako v0.1.0

- [ ] Isti commit je prestal CI in ima pregledane artefakte za tri OS.
- [ ] Paketi so ročno zagnani na Windows/macOS/Linux; offline pregled, uvoz/izvoz in recheck delujejo.
- [ ] Učitelj in dva udeleženca so potrdili soglasje, obnovo dostopa, osveževanje, varnost, izbris in potek.
- [ ] Namestitev Git je bila posebej ročno potrjena in preverjena; avtomatizirani testi niso nameščali paketov.
- [ ] Šola je pregledala gostovanje in podatkovne prakse.
- [ ] Uporabnik je izrecno dovolil oznako; javna izdaja zahteva ločeno odločitev.
