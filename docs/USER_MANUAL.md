# Nostr Nations User Manual

Welcome to **Nostr Nations**, a turn-based strategy game where you build civilizations, explore new lands, and compete with other players for dominance. This guide will help you get started and master the game.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Main Menu](#main-menu)
3. [Gameplay Basics](#gameplay-basics)
4. [Units](#units)
5. [Cities](#cities)
6. [Research & Technology](#research--technology)
7. [Diplomacy](#diplomacy)
8. [Victory Conditions](#victory-conditions)
9. [Multiplayer](#multiplayer)
10. [Tips & Strategies](#tips--strategies)

---

## Getting Started

### System Requirements

**Minimum Requirements:**

- Modern web browser (Chrome, Firefox, Safari, Edge)
- 4GB RAM
- Stable internet connection for multiplayer

**Recommended:**

- 8GB RAM
- Hardware-accelerated graphics enabled in browser
- Broadband internet connection

### Installation

Nostr Nations runs in your web browser - no installation required!

1. Navigate to the game URL in your browser
2. Allow the game to load completely
3. For the best experience, use fullscreen mode (press `F11` or use browser menu)

**Optional:** Add the game to your home screen on mobile devices for an app-like experience.

### First Launch

When you first open Nostr Nations:

1. The game will load assets and initialize
2. You'll see the main menu with options to start playing
3. If this is your first time, you'll be prompted to create or connect a Nostr identity

### Creating a Nostr Identity

Nostr Nations uses **Nostr** for player identity and multiplayer connectivity.

**If you're new to Nostr:**

1. Select "Create New Identity" when prompted
2. The game will generate a keypair for you
3. **Important:** Save your private key securely - this is your account!
4. Your public key becomes your player ID

**If you have an existing Nostr identity:**

1. Select "Use Existing Identity"
2. Enter your private key (nsec) or use a Nostr browser extension
3. Your identity will be linked to your game profile

> **Security Note:** Never share your private key with anyone. The game only needs it locally to sign messages.

---

## Main Menu

### New Game

Start a fresh game with customizable settings:

- **Map Size:** Small, Medium, Large, or Huge - affects game length and player count
- **Map Type:** Continents, Pangaea, Islands, or Random
- **Number of AI Players:** Choose how many computer opponents to face
- **Difficulty:** Settler (easy) through Deity (expert)
- **Game Speed:** Quick, Standard, or Epic - affects how long each era lasts
- **Victory Conditions:** Enable or disable specific victory types

After configuring, select "Start Game" to begin.

### Load Game

Continue a previously saved game:

1. Select "Load Game" from the main menu
2. Browse your saved games (sorted by date)
3. Select a save file to see details (turn number, date saved)
4. Click "Load" to continue playing

Saves are stored locally in your browser. Use the export feature to back up important saves.

### Join Game

Join a multiplayer game hosted by another player:

**Via QR Code:**

1. Select "Join Game"
2. Choose "Scan QR Code"
3. Point your camera at the host's QR code
4. Confirm the game details and join

**Via Ticket Entry:**

1. Select "Join Game"
2. Choose "Enter Ticket"
3. Type or paste the game ticket provided by the host
4. Confirm and join the game

### Settings

Customize your experience:

**Audio:**

- Master Volume
- Music Volume
- Sound Effects Volume
- Toggle ambient sounds

**Graphics:**

- Animation quality (Low/Medium/High)
- Show grid overlay
- Unit animations on/off
- Particle effects

**Gameplay:**

- Auto-end turn when no actions remain
- Show yield icons on map
- Confirmation prompts for combat
- Quick combat (skip animations)

### Exit

Close the game. In browser, this returns you to the previous page. Your current game state is auto-saved.

---

## Gameplay Basics

### Understanding the Map

The game world is made up of hexagonal tiles, each with unique properties:

**Terrain Types:**

- **Grassland:** High food yield, good for cities
- **Plains:** Balanced food and production
- **Desert:** Low yields, but may contain valuable resources
- **Tundra:** Limited food, found in cold regions
- **Snow:** Very low yields, difficult to settle
- **Hills:** Bonus production, defensive advantage
- **Mountains:** Impassable, but provide defense to adjacent tiles
- **Coast/Ocean:** Enable naval units and sea trade

**Features:**

- **Forests:** Provide production, can be cleared
- **Jungles:** Provide food, slow movement
- **Rivers:** Fresh water for cities, defensive bonus
- **Oases:** High yields in desert

**Resources:**

- **Bonus Resources:** Increase tile yields (wheat, cattle, fish)
- **Strategic Resources:** Required for certain units (iron, horses, oil)
- **Luxury Resources:** Boost happiness and can be traded

### Camera Controls

Navigate the map with ease:

| Action         | Mouse             | Keyboard           |
| -------------- | ----------------- | ------------------ |
| Pan            | Click and drag    | Arrow keys or WASD |
| Zoom In        | Scroll up         | + or =             |
| Zoom Out       | Scroll down       | -                  |
| Center on Unit | Double-click unit | Spacebar           |
| Center on City | Double-click city | C                  |

### Selecting Units and Cities

- **Left-click** a unit or city to select it
- **Right-click** with a unit selected to move or attack
- Hold **Shift** to queue multiple orders
- Press **Tab** to cycle through your units
- Press **Escape** to deselect

### The Turn System

Nostr Nations is turn-based:

1. At the start of your turn, all your units gain movement points
2. Give orders to your units (move, attack, build, etc.)
3. Manage your cities (adjust production, assign citizens)
4. Set research and diplomacy as needed
5. Click **"End Turn"** or press **Enter** when finished

The game will prompt you if units still have actions available.

---

## Units

### Unit Types

**Military Units:**

- **Melee:** Warriors, Swordsmen, Infantry - attack adjacent enemies
- **Ranged:** Archers, Crossbowmen, Artillery - attack from distance
- **Mounted:** Horsemen, Knights, Cavalry - fast movement
- **Siege:** Catapults, Trebuchets, Cannons - effective against cities
- **Naval:** Galleys, Frigates, Battleships - control the seas

**Civilian Units:**

- **Settlers:** Found new cities (consumed when used)
- **Workers:** Improve tiles with farms, mines, roads
- **Traders:** Establish trade routes between cities

### Moving Units

1. Select a unit by clicking on it
2. Valid movement tiles are highlighted
3. Right-click a destination to move
4. Units with remaining movement can continue moving

**Movement Costs:**

- Flat terrain: 1 movement point
- Hills/Forest: 2 movement points
- Roads: 0.5 movement points
- Rivers: Costs all remaining movement to cross (without bridge)

### Combat Basics

When your unit attacks:

1. Select your military unit
2. Enemy units in range are highlighted in red
3. Hover over enemies to see combat preview (estimated damage)
4. Right-click to attack

**Combat Factors:**

- Unit strength and health
- Terrain bonuses (hills, rivers, fortifications)
- Unit promotions and special abilities
- Support from adjacent friendly units

### Fortifying and Healing

- **Fortify:** Press `F` to fortify a unit in place. Fortified units gain defensive bonuses and will remain stationary until activated.
- **Sleep:** Press `S` to put a unit to sleep. They won't ask for orders until an enemy approaches.
- **Healing:** Units heal automatically each turn when not in combat. Healing is faster in friendly territory and fastest in cities.

### Promotions

Units gain experience (XP) from combat. With enough XP, they can earn promotions:

1. A star icon appears when a promotion is available
2. Click the unit and select "Promote"
3. Choose from available promotions based on unit type

**Example Promotions:**

- **Shock:** Bonus vs. melee units
- **Drill:** Bonus vs. ranged units
- **March:** Heal even after moving
- **Blitz:** Attack twice per turn

---

## Cities

### Founding Cities

To found a new city:

1. Train a **Settler** unit in an existing city
2. Move the Settler to a desirable location
3. Press `B` or click "Found City"
4. Name your new city

**Good City Locations:**

- Near fresh water (rivers, lakes)
- On or adjacent to luxury/strategic resources
- Mix of terrain for balanced yields
- Defensible positions (hills, between mountains)

### City Growth and Food

Cities grow when they accumulate enough food:

- Each citizen requires 2 food per turn
- Excess food fills the growth meter
- When the meter is full, population increases by 1
- More citizens means more tiles can be worked

**Managing Growth:**

- Assign citizens to high-food tiles for faster growth
- Build farms and granaries to boost food production
- Balance growth with production needs

### Production Queue

Each city produces one item at a time:

1. Open a city screen by double-clicking the city
2. Select what to build from the production menu
3. Items are built when enough production accumulates
4. Queue multiple items by Shift-clicking

**Production Options:**

- Units (military and civilian)
- Buildings (city improvements)
- Wonders (powerful unique buildings)
- Projects (special city actions)

### Buildings and Their Effects

Buildings improve your cities:

| Building   | Effect                         |
| ---------- | ------------------------------ |
| Granary    | +2 Food, food stored on growth |
| Library    | +1 Science per 2 citizens      |
| Barracks   | New units start with XP        |
| Market     | +25% Gold in city              |
| Walls      | +5 City Defense                |
| Workshop   | +2 Production                  |
| University | +50% Science in city           |

Each building has a maintenance cost in gold per turn.

### Citizens and Specialists

Citizens work tiles around your city or serve as specialists:

**Tile Workers:**

- Automatically assigned to best available tiles
- Can be manually reassigned in city screen
- Each citizen works one tile

**Specialists:**

- Assign citizens to specialist slots in buildings
- **Scientists:** Generate extra research
- **Merchants:** Generate extra gold
- **Engineers:** Generate extra production
- **Artists:** Generate culture and great person points

---

## Research & Technology

### Using the Tech Tree

Access the tech tree by clicking the research icon or pressing `T`:

- Technologies are arranged by era (Ancient, Classical, Medieval, etc.)
- Lines show prerequisites - you must research earlier techs first
- Completed techs are highlighted
- Your current research shows progress

### Choosing Research

1. Open the tech tree
2. Click on a technology you want to research
3. If prerequisites aren't met, the game shows the optimal path
4. Research progresses each turn based on your science output

**Research Tips:**

- Focus on technologies that unlock resources you have
- Military techs when facing aggression
- Economic techs when at peace
- Balance immediate needs with long-term goals

### Technology Benefits

Technologies unlock:

- **New Units:** Better military options
- **New Buildings:** City improvements
- **Tile Improvements:** Better yields from workers
- **Abilities:** New diplomatic options, government types
- **Wonders:** Powerful unique buildings
- **Resources:** Ability to use strategic resources

---

## Diplomacy

### Meeting Other Players

You'll encounter other civilizations through:

- Exploring and finding their units/cities
- Being contacted by them first
- Random events and trade routes

Once met, civilizations appear in your diplomacy screen.

### Relationship Status

Your standing with each civilization:

- **Friendly:** Positive relations, likely to cooperate
- **Neutral:** No strong feelings either way
- **Unfriendly:** Tensions exist, be cautious
- **Hostile:** War is likely imminent
- **At War:** Open conflict

Relationships change based on your actions, agreements, and competing interests.

### Declaring War and Making Peace

**Declaring War:**

1. Open the diplomacy screen
2. Select the target civilization
3. Choose "Declare War"
4. Confirm your decision

> **Warning:** Breaking treaties or surprise attacks harm your reputation with all players.

**Making Peace:**

1. Open diplomacy with your enemy
2. Select "Negotiate Peace"
3. Propose or accept terms
4. Both parties must agree

### Treaties

Formalize relationships with agreements:

- **Open Borders:** Units can pass through each other's territory
- **Defensive Pact:** Automatic war declaration if either is attacked
- **Research Agreement:** Both players gain science bonus
- **Alliance:** Full military and economic cooperation

Treaties last a set number of turns and can be renewed.

### Trade Deals

Exchange resources and gold:

1. Open diplomacy with another player
2. Select "Trade"
3. Add items to your offer and their offer
4. Adjust until both sides agree
5. Confirm the deal

**Tradeable Items:**

- Luxury resources
- Strategic resources
- Gold (lump sum or per turn)
- Open Borders
- Maps

---

## Victory Conditions

### Domination Victory

**Objective:** Capture all original capital cities.

- Each civilization starts with a capital (marked with a star)
- Capture enemy capitals through conquest
- Control all capitals to win
- You don't need to eliminate entire civilizations, just take their capitals

### Science Victory

**Objective:** Complete the space program and launch a spacecraft.

1. Research all required space technologies
2. Build spaceship parts in your cities:
   - SS Booster
   - SS Cockpit
   - SS Engine
   - SS Stasis Chamber
3. Assemble the spaceship
4. Launch to win!

### Economic Victory

**Objective:** Accumulate massive wealth and fund a world project.

1. Build your economy through trade and city development
2. Accumulate the required treasury (scales with map size)
3. Fund the World Bank wonder
4. Maintain economic dominance for 10 turns to win

### Diplomatic Victory

**Objective:** Be elected World Leader by the global community.

1. Build the United Nations wonder (or wait for another to build it)
2. Improve relationships with other civilizations
3. Gain votes through alliances and city-states
4. Win the World Leader election
5. Requires majority vote from all players

### Score Victory

**Objective:** Have the highest score when time runs out.

If no civilization achieves another victory by the final turn, scores are calculated:

- Population
- Territory size
- Technologies researched
- Wonders built
- Military strength

The highest score wins!

---

## Multiplayer

### Hosting a Game

Create a multiplayer game for others to join:

1. Select "New Game" from main menu
2. Choose your game settings
3. Enable "Multiplayer" option
4. Set player slots (human vs AI)
5. Click "Create Game"
6. Share the QR code or ticket with friends

### Joining via QR Code

The fastest way to join a friend's game:

1. Ask the host to display their game's QR code
2. Select "Join Game" > "Scan QR Code"
3. Scan the code with your device
4. Review game settings and confirm
5. Wait in lobby until host starts

### Turn Timing

Multiplayer games can use different turn modes:

- **Simultaneous Turns:** All players take turns at once (faster)
- **Sequential Turns:** Players take turns one at a time (more strategic)
- **Turn Timer:** Optional time limit per turn (keeps game moving)

The host configures these settings before starting.

### Connection Issues

If you experience connection problems:

- **Check your internet connection** - ensure you have stable connectivity
- **Refresh the page** - the game will attempt to reconnect automatically
- **Rejoin the game** - use the same ticket/QR code if disconnected
- **Contact the host** - they may need to pause or adjust settings

Games auto-save frequently, so progress should not be lost.

---

## Tips & Strategies

### Early Game Priorities

1. **Explore immediately** - send your starting Warrior to find nearby resources and meet neighbors
2. **Build a Scout or second Warrior** for faster exploration
3. **Found 2-3 cities** before turn 50 to establish a strong base
4. **Secure luxury resources** to keep your population happy
5. **Research Pottery early** for Granaries and faster growth

### Balancing Expansion and Defense

- **Don't over-expand** - each city increases costs and spreads your defenses thin
- **Build military as you grow** - one defender per city minimum
- **Walls are worth it** - especially on frontier cities
- **Use terrain** - settle on hills, near rivers, or between mountains
- **Watch your neighbors** - if they're building armies, you should too

### When to Go to War

**Good reasons to attack:**

- They have a crucial strategic resource you need
- They're close to winning and must be stopped
- They're weak and you can gain cities easily
- They attacked your ally

**Think twice if:**

- You're not prepared (military, economy, tech)
- Multiple civilizations will turn against you
- It will distract from a victory you're close to achieving
- The target has powerful allies

**During war:**

- Focus on capturing cities, not destroying units
- Use ranged units to soften defenses before melee attacks
- Pillage enemy improvements to hurt their economy
- Know when to negotiate peace - sometimes taking 2 cities is better than grinding on

---

## Keyboard Shortcuts Reference

| Key      | Action                     |
| -------- | -------------------------- |
| Enter    | End Turn                   |
| Escape   | Deselect / Cancel          |
| Tab      | Next Unit                  |
| Spacebar | Center on Active Unit      |
| T        | Tech Tree                  |
| C        | City Screen (nearest city) |
| M        | Toggle Mini-map            |
| F        | Fortify Unit               |
| S        | Sleep Unit                 |
| B        | Found City (Settler)       |
| P        | Build Improvement (Worker) |
| Delete   | Disband Unit               |
| F1       | Help                       |

---

**Good luck, and may your nation thrive!**
