# Podatki in zasebnost

Lokalna diagnostika deluje brez računa ali strežnika. Desktop privzeto ne izvaja HTTPS preverjanj. Ob vklopu HTTPS testov ciljni strežnik vidi IP naslov; Preflight mu ne pošilja rezultatov.

API sob hrani validirani profil, naključno kodo sobe, čas nastanka/poteka in hash učiteljskega žetona. Seja hrani naključni ID, ustvarjen vzdevek, hash lastnega žetona, čas posodobitve in samo izrecno deljene minimalne rezultate (ID preverjanja, status, normalizirana različica, reason_code). Ne zahtevamo imen, e-pošte ali računa. Ne pošiljaj osebnih podatkov v profilih ali ID-jih. Strežnik teh uporabniško vnesenih besedil ne more samodejno prepoznati kot osebnih podatkov.

PUT rezultatov zahteva consent: true. V fazi 05 še ni povezave z namizno aplikacijo ali učiteljskega UI; to pride v fazi 06. Odjemalec mora pred deljenjem pokazati podatke in pridobiti izrecno soglasje. Preklic je DELETE lastne seje z njenim žetonom. Učitelj izbriše sobo in vse seje s svojim žetonom. Žetoni se vrnejo samo ob ustvarjanju; izgubljenega žetona ni mogoče obnoviti.

Privzeta hramba je največ 24 ur, nastavljiva na 60–86400 sekund. Dostop po poteku ni dovoljen; fizični SQL DELETE se izvede ob API dostopu ali vsako minuto, ob ponovnem zagonu pa takoj. Izbris iz SQLite ne jamči forenzičnega izbrisa iz prostih strani diska, varnostnih kopij ali posnetkov. Upravljavec mora omejiti dostop do diska in urediti hrambo kopij.

Aplikacija nima dnevnika IP naslovov, vsebine zahtev ali žetonov. Reverse proxy in ponudnik gostovanja lahko kljub temu beležita IP-je ter URL-je: izključi query stringe, Authorization in vsebino zahtev iz logov, določi kratko hrambo in ustrezno obvestilo uporabnikom. HTTPS in strežniško gostovanje lahko vključujeta osebne podatke. Projekt sam po sebi ni zagotovilo skladnosti z GDPR; šola mora določiti pravno podlago, odgovornosti, gostovanje in hrambo.
