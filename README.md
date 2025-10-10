# 2D orthogonal knapsack problem with rotation (2D-OKP-R)

Ciljna ocena: 10

Student: SV54-2021 Ivanov Maša

### Opis problema

Problem se sastoji iz postavljanja pravougaonika dimenzija (w,h) na pravougaonu površinu dimenzija (W,H). Pravougaonici koji se postavljaju ne smeju da se preklapaju i dozvoljeno je njihovo rotiranje za 90 stepeni. Cilj je da se postave pravougaonici tako da ostane što manje slobodnog mesta na površini. Postavljanje svih pravougaonika nije obavezno. 

### Doprinos

Cilj rada je razvoj *open source* biblioteke u *Rust*-u koja može da posluži kao osnova za dalja istraživanja i primene u oblasti 2D optimizacije sečenja i pakovanja (*cutting & packing*). Za razliku od postojećih implementacija, postoji mogućnost eksplicitnog odabira heuristike i poređenja performansi različitih heurističkih pristupa. Pored toga, biblioteka obezbeđuje metrike za evaluaciju (kao što su iskorišćenost materijala, broj iskorišćenih pravougaonika i vreme izvršavanja), čime se olakšava upoređivanje i analiza rezultata.

### Metode rešavanja

Rešenje je implementirano u programskom jeziku *Rust*. Za implementaciju se koristi **evolutivni algoritam**. 

**Hromozom** je predstavljen nizom jedinica i nula, od kojih svake dve cifre predstavljaju jedan pravougaonik:
* Prva cifra predstavlja redni broj pravougaonika 
* Druga cifra predstavlja da li je pravougaonik rotiran (1 da, 0 ne)

Korišćena **heuristika** za dekodiranje hromozoma je *MaxRects* - *Best Area Fit* varijanta. Vodi spisak najvećih pravougaonika slobodnog prostora koji ažurira svaki put kada postavi neki pravougaonik. 

*Fitness* funkcija je procenat iskorišćene površine. 

**Paralelizovani** delovi genetskog algoritma:
* dekodiranje hromozoma
* *crossover* (opciono, po potrebi)
* funkcija mutacije (opciono, po potrebi)
 
### Vizuelizacija ispisa 
Biblioteka: `macroquad`

Konačno rešenje, na kom je predstavljen optimalan raspored bi bilo vizuelno prikazano. 

### Performanse
Posmatraju se vreme izvršavanja algoritma i količine *waste*-a u procentima, koji se dalje porede sa optimalnim rezultatima za zadate vrednosti materijala i pravougaonika. Ulazi koji se testiraju su *gcut1r*-*gcut17r*.

### Diplomski deo
Implementacija dodatnih heuristika za raspoređivanje:
* *Skyline*, razlikuje se od *MaxRect* po tome što pravougaonike postavlja odozdo ka gore minimizujući ukupnu visinu postavljenih pravougaonika
* *Guillotine*, razlikuje se od *MaxRect* po tome što deli slobodan prostor pravim linijama koje se pružaju do ivice materijala

Izvršava se evaluacija rezultata heuristika po metrici procenat otpada i vreme izvršavanja i porede se rezultati. 

Poređenje se dodatno vizuelno prikazuje graficima koji ilustruju odnos optimalnog rezultata od dobijenog za svaki gcut problem, za sve heuristike (graf po heuristici).

