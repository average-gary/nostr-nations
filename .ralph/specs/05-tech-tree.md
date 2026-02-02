# Nostr Nations - Technology Tree

## Overview

The technology tree spans 6 eras with 60+ technologies, unlocking units, buildings, wonders, and game mechanics as civilizations progress through history.

## Era Progression

| Era | Approx. Techs | Themes |
|-----|---------------|--------|
| Ancient | 1-10 | Agriculture, Bronze, Writing |
| Classical | 11-20 | Iron, Philosophy, Engineering |
| Medieval | 21-32 | Feudalism, Gunpowder, Universities |
| Renaissance | 33-42 | Exploration, Printing, Banking |
| Industrial | 43-52 | Steam, Electricity, Mass Production |
| Modern | 53-60 | Computers, Nuclear, Space |

## Technology Research

### Science Generation

Science per turn from:
- Population: 1 science per citizen
- Buildings: Libraries, Universities, etc.
- Specialists: Scientists (+3 each)
- Trade routes: Science agreements
- Great Scientists: Bulb for instant science

### Research Cost

```
base_cost = era_base * (1 + 0.1 * num_cities)

Era bases:
- Ancient: 50
- Classical: 100
- Medieval: 200
- Renaissance: 400
- Industrial: 800
- Modern: 1600
```

### Research Overflow

Excess science carries over to next technology.

## Complete Technology Tree

### Ancient Era

```
                    Agriculture
                         |
            ┌────────────┼────────────┐
            ▼            ▼            ▼
        Pottery      Animal        Mining
            |       Husbandry        |
            ▼            |           ▼
        Writing     ────►│◄────  Masonry
            |            |           |
            ├────────────┤           |
            ▼            ▼           ▼
       Calendar      Trapping     Bronze
            |            |        Working
            └────────────┼───────────┘
                         ▼
                   The Wheel
                         |
            ┌────────────┼────────────┐
            ▼            ▼            ▼
       Horseback    Archery      Sailing
        Riding
```

#### Ancient Technologies

| Tech | Cost | Prerequisites | Unlocks |
|------|------|---------------|---------|
| Agriculture | 20 | Start | Farm improvement, Settler |
| Pottery | 35 | Agriculture | Granary, Shrine |
| Animal Husbandry | 35 | Agriculture | Pasture, Scout |
| Mining | 35 | Agriculture | Mine improvement, Quarry |
| Writing | 55 | Pottery | Library, Open Borders |
| Masonry | 55 | Mining | Walls, Quarry, Pyramids |
| Calendar | 70 | Pottery | Plantation, Stonehenge |
| Trapping | 55 | Animal Husbandry | Camp improvement, Trading Post |
| Bronze Working | 55 | Mining | Spearman, Barracks, Colossus |
| The Wheel | 55 | Animal Husbandry + Mining | Chariot Archer, Roads, Water Mill |
| Horseback Riding | 75 | The Wheel | Horseman, Stable |
| Archery | 35 | Agriculture | Archer, Temple of Artemis |
| Sailing | 55 | Pottery | Work Boat, Galley, Lighthouse |

### Classical Era

```
      Writing              Bronze Working
         |                       |
         ▼                       ▼
    Philosophy              Iron Working
         |                       |
    ┌────┴────┐             ┌────┴────┐
    ▼         ▼             ▼         ▼
  Drama    Mathematics   Construction  Metal
   Poetry        |            |       Casting
    |            └─────┬──────┘          |
    ▼                  ▼                 ▼
 Theology          Engineering        Currency
    |                  |                 |
    └─────────────────┬──────────────────┘
                      ▼
                   Civil
                  Service
```

#### Classical Technologies

| Tech | Cost | Prerequisites | Unlocks |
|------|------|---------------|---------|
| Philosophy | 100 | Writing | National College, Great Library |
| Iron Working | 150 | Bronze Working | Swordsman, Colosseum |
| Drama & Poetry | 175 | Philosophy | Amphitheater, Great Artists |
| Mathematics | 100 | The Wheel | Catapult, Courthouse |
| Construction | 100 | Masonry | Colosseum, Circus, Lumber Mill |
| Metal Casting | 120 | Bronze Working | Forge, Workshop |
| Theology | 200 | Philosophy | Temple, Monastery, Hagia Sophia |
| Engineering | 175 | Mathematics + Construction | Aqueduct, Great Wall, Fort |
| Currency | 175 | Mathematics | Market, Mint, Petra |
| Civil Service | 275 | Theology + Currency | Pikeman, Chichen Itza |

### Medieval Era

```
    Theology         Engineering         Currency
        |                 |                  |
        ▼                 ▼                  ▼
     Education         Physics           Guilds
        |                 |                  |
        └────────┬────────┴──────────────────┘
                 ▼                           |
              Steel                          |
                 |                           ▼
        ┌────────┴────────┐              Banking
        ▼                 ▼                  |
    Machinery         Printing              |
        |               Press               |
        └────────┬────────┴──────────────────┘
                 ▼
            Gunpowder
                 |
        ┌────────┴────────┐
        ▼                 ▼
    Chemistry         Astronomy
```

#### Medieval Technologies

| Tech | Cost | Prerequisites | Unlocks |
|------|------|---------------|---------|
| Education | 275 | Theology | University, Oxford University |
| Physics | 275 | Engineering | Trebuchet, Notre Dame |
| Guilds | 275 | Currency | Workshop upgrades, Machu Picchu |
| Steel | 325 | Metal Casting | Longswordsman |
| Banking | 400 | Guilds + Education | Bank, Forbidden Palace |
| Machinery | 400 | Steel + Guilds | Crossbowman, Ironworks |
| Printing Press | 400 | Machinery + Physics | Public School |
| Gunpowder | 475 | Steel + Physics | Musketman, Himeji Castle |
| Chemistry | 550 | Gunpowder | Cannon, Dye works |
| Astronomy | 550 | Education + Physics | Observatory, Caravel, Leaning Tower |

### Renaissance Era

```
    Gunpowder         Astronomy          Banking
        |                 |                 |
        ▼                 ▼                 ▼
    Metallurgy        Navigation       Economics
        |                 |                 |
        └────────┬────────┴─────────────────┘
                 ▼                          |
             Acoustics                      |
                 |                          ▼
                 ▼                    Scientific
              Architecture               Theory
                 |                          |
        ┌────────┴────────┐                 |
        ▼                 ▼                 |
    Military          Fertilizer           |
    Science               |                |
        └────────┬────────┴─────────────────┘
                 ▼
             Biology
```

#### Renaissance Technologies

| Tech | Cost | Prerequisites | Unlocks |
|------|------|---------------|---------|
| Metallurgy | 650 | Gunpowder | Lancer, Forge upgrade |
| Navigation | 650 | Astronomy | Frigate, Harbor, Great Lighthouse |
| Economics | 650 | Banking + Printing Press | Windmill, Big Ben |
| Acoustics | 750 | Education + Astronomy | Opera House, Sistine Chapel |
| Scientific Theory | 850 | Astronomy | Public School, Porcelain Tower |
| Architecture | 850 | Acoustics + Economics | Palace upgrade, Louvre |
| Military Science | 850 | Economics + Metallurgy | Cavalry, Military Academy |
| Fertilizer | 950 | Chemistry + Scientific Theory | Farm upgrades |
| Biology | 950 | Fertilizer + Scientific Theory | Hospital, Clinic |

### Industrial Era

```
    Military          Biology          Architecture
    Science             |                   |
        |               ▼                   ▼
        ▼          Electricity        Industrialization
    Dynamite            |                   |
        |               └─────────┬─────────┘
        ▼                         ▼
    Combustion                 Telegraph
        |                         |
        └─────────────┬───────────┘
                      ▼
                    Radio
                      |
         ┌────────────┼────────────┐
         ▼            ▼            ▼
    Replaceable    Flight      Refrigeration
      Parts
```

#### Industrial Technologies

| Tech | Cost | Prerequisites | Unlocks |
|------|------|---------------|---------|
| Industrialization | 1150 | Scientific Theory | Factory, Coalplant |
| Electricity | 1150 | Biology | Stock Exchange, Hydro Plant |
| Dynamite | 950 | Fertilizer + Military Science | Artillery |
| Telegraph | 1150 | Electricity + Industrialization | Military Base |
| Combustion | 1350 | Dynamite | Landship, Oil Well |
| Radio | 1450 | Telegraph | Broadcast Tower, Eiffel Tower |
| Replaceable Parts | 1450 | Radio | Infantry, Statue of Liberty |
| Flight | 1450 | Radio | Fighter, Bomber, Pentagon |
| Refrigeration | 1450 | Electricity | Stadium, Food processing |

### Modern Era

```
    Replaceable        Flight         Refrigeration
       Parts             |                 |
         |               ▼                 ▼
         ▼           Advanced           Ecology
       Plastics       Flight              |
         |               |                |
         └───────┬───────┴────────────────┘
                 ▼
              Computers
                 |
         ┌───────┴───────┐
         ▼               ▼
       Robotics       Telecommunications
         |               |
         └───────┬───────┘
                 ▼
              Lasers
                 |
         ┌───────┴───────┐
         ▼               ▼
       Nuclear        Stealth
       Fission       Technology
         |               |
         ▼               ▼
       Nuclear        Advanced
       Fusion         Ballistics
         |               |
         └───────┬───────┘
                 ▼
           Particle
           Physics
                 |
                 ▼
            Nanotechnology
                 |
                 ▼
           Future Tech
```

#### Modern Technologies

| Tech | Cost | Prerequisites | Unlocks |
|------|------|---------------|---------|
| Plastics | 1650 | Replaceable Parts + Combustion | Research Lab |
| Advanced Flight | 1650 | Flight + Radio | Jet Fighter, Paratrooper |
| Ecology | 1650 | Refrigeration | Solar Plant, Recycling Center |
| Computers | 1850 | Plastics + Advanced Flight | Carrier, SS Cockpit |
| Robotics | 1850 | Computers | Mech Infantry, Spaceship Factory |
| Telecommunications | 1850 | Computers | Nuclear Submarine |
| Lasers | 2050 | Robotics + Telecommunications | Modern Armor, SS Engine |
| Nuclear Fission | 2050 | Lasers | Atomic Bomb, Nuclear Plant |
| Stealth Technology | 2050 | Lasers | Stealth Bomber |
| Nuclear Fusion | 2250 | Nuclear Fission | Giant Death Robot, SS Stasis |
| Advanced Ballistics | 2250 | Stealth Technology | ICBM, SS Booster |
| Particle Physics | 2500 | Nuclear Fusion + Advanced Ballistics | GDR upgrades |
| Nanotechnology | 2750 | Particle Physics | All unit healing +25 |
| Future Tech | 3000 | Nanotechnology | +5% all yields (repeatable) |

## Technology Agreements

### Research Agreements

Two players can sign research agreement:
- Both pay gold upfront
- After 30 turns, both receive science boost
- Boost = 50% of cheapest unknown tech

### Technology Trading

Players can exchange:
- Known technologies
- Gold for technology
- Technology for other diplomatic items

## Great Scientists

### Generation

Great Scientists born from:
- Science specialist slots
- Wonders (Great Library, etc.)
- Random events

### Abilities

1. **Academy**: +8 science tile improvement
2. **Bulb**: Instant science = 8 turns of current output
3. **Golden Age**: Trigger 8-turn golden age (shared with other Great People)
