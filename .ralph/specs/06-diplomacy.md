# Nostr Nations - Diplomacy System

## Overview

The diplomacy system governs interactions between 2-4 players, including treaties, trade, alliances, and warfare. All diplomatic actions are recorded as Nostr events for transparency and verification.

## Diplomatic States

### Relationship Levels

| State | Description | Allowed Actions |
|-------|-------------|-----------------|
| War | Active conflict | Combat, pillaging, city capture |
| Hostile | Tense relations | Limited trade, movement restrictions |
| Neutral | Default state | Trade, open borders negotiable |
| Friendly | Good relations | Trade bonuses, open borders likely |
| Allied | Full partnership | Shared visibility, defensive pacts |

### State Transitions

```
         ┌──────────────────────────────────┐
         │                                  │
         ▼                                  │
       WAR ◄────────────────────────┐      │
         │                          │      │
         │ Peace Treaty             │ Declare War
         ▼                          │      │
     HOSTILE ◄──────────────────────┤      │
         │                          │      │
         │ Improved Relations       │ Denounce
         ▼                          │      │
     NEUTRAL ◄──────────────────────┤      │
         │                          │      │
         │ Trade/Treaties           │ Broken Treaty
         ▼                          │      │
    FRIENDLY ◄──────────────────────┤      │
         │                          │      │
         │ Defensive Pact           │ Betrayal
         ▼                          │      │
     ALLIED ────────────────────────┴──────┘
```

## Treaties and Agreements

### Open Borders

Allow units to pass through territory:
- Duration: 30 turns
- Can be mutual or one-way
- Costs: Usually gold or reciprocal
- Breaking: End treaty, units expelled

### Defensive Pact

Mutual defense agreement:
- Duration: Permanent until cancelled
- Effect: Declaring war on one = declaring war on both
- Requires: Friendly or Allied status
- Cooldown: 10 turns after cancellation

### Research Agreement

Joint scientific cooperation:
- Cost: Gold upfront (scales with era)
- Duration: 30 turns
- Reward: Both receive tech boost at end
- Cancelled: If war declared, gold lost

### Trade Agreement

Economic cooperation:
- Resource trades
- Gold per turn
- Lump sum gold
- Technologies (if enabled)
- Duration: 30 turns for GPT deals

### Non-Aggression Pact

Promise not to attack:
- Duration: 30 turns
- Breaking: Severe diplomatic penalty
- Requires: Neutral or better relations

### Military Access

Allow military units in territory:
- Separate from Open Borders
- Usually paired with Defensive Pact
- Useful for coordinated attacks on third party

## War and Peace

### Declaring War

Process:
1. Player initiates war declaration (Nostr event)
2. War state immediately active
3. Units can now attack
4. Trade agreements terminated
5. Open borders revoked
6. Penalty with other players (warmonger score)

**Warmonger Penalties:**
- Declaring war: -10 with all players
- Capturing cities: -5 per city
- Eliminating player: -20

### Casus Belli (Just War Reasons)

Declaring war with valid reason reduces penalty:

| Casus Belli | Condition | Penalty Reduction |
|-------------|-----------|-------------------|
| Formal War | After denouncing | 50% |
| Defensive War | Responding to attack | 100% |
| Liberation | Recapturing own city | 100% |
| Protectorate | Defending city-state (if any) | 75% |
| Territorial | Settling near your borders | 25% |

### Peace Negotiations

Peace treaty terms can include:
- Cities: Transfer of cities
- Gold: Lump sum payment
- Gold per Turn: Ongoing payment
- Resources: Luxury/strategic resources
- Open Borders: As part of peace
- Cede Territory: Tiles without cities

**Minimum War Duration:** 10 turns before peace allowed

### Armistice

Temporary ceasefire:
- Duration: 10 turns
- No combat, but war state continues
- Used for negotiations
- Can be broken (additional penalty)

## Trade System

### Tradeable Items

| Category | Items | Notes |
|----------|-------|-------|
| Resources | Luxury, Strategic | Per turn for duration |
| Gold | Lump Sum | Immediate transfer |
| Gold/Turn | GPT | Over treaty duration |
| Technologies | Any known tech | If trading enabled |
| Cities | Own cities | With population |
| Maps | Explored territory | One-time reveal |
| Treaties | Open Borders, etc. | Bundled with trade |

### Trade Deal Interface

Proposal structure:
```
Player A Offers:
- 10 Gold per Turn
- Open Borders
- Silk (Luxury)

Player A Requests:
- Iron (Strategic)
- 100 Gold (Lump Sum)
```

### Fair Trade Calculation

AI-assisted fairness indicator:
- Green: Fair deal
- Yellow: Slightly unbalanced
- Red: Very unbalanced

Based on:
- Current market values
- Relationship status
- Strategic importance

## Communication

### Diplomatic Messages

Structured message types:
- **Proposal**: Treaty or trade offer
- **Counter-Proposal**: Modified offer
- **Acceptance**: Accept current terms
- **Rejection**: Decline with optional reason
- **Denouncement**: Public condemnation
- **Praise**: Public endorsement

### Private vs Public

- **Private Messages**: NIP-04 encrypted, only participants see
- **Public Statements**: Visible to all players
- **Denouncements**: Always public

### Message Nostr Events

```json
{
  "kind": 30140,
  "tags": [
    ["g", "<game-id>"],
    ["p", "<prev-event>"],
    ["to", "<recipient-pubkey>"],
    ["type", "proposal|acceptance|rejection|denounce"],
    ["visibility", "private|public"]
  ],
  "content": "<encrypted-or-plain-message>"
}
```

## Diplomatic Victory

### United Nations

Building the UN wonder enables diplomatic victory:
- Available in Modern era
- Requires: Telecommunications technology
- Requires: All living players' agreement to build

### World Leader Election

Every 20 turns after UN built:
1. All players vote
2. Votes weighted by:
   - Base: 1 vote per player
   - Population: +1 per 10 citizens
   - City-States (if any): +1 per ally
3. Need majority (>50%) to win
4. If no winner, election continues

### Diplomatic Influence

Ways to gain votes:
- High relationship with other players
- Trade benefits provided
- Liberation of cities
- Gifts and aid

## Multiplayer Diplomacy Rules

### 2-Player Games

Simplified diplomacy:
- No alliances (only two players)
- Trade still possible
- War/Peace mechanics same
- Diplomatic victory disabled (or modified)

### 3-4 Player Games

Full diplomacy:
- Alliances matter
- Kingmaker scenarios possible
- Defensive pacts can shift balance
- Diplomatic victory viable

### Turn Timer Considerations

Diplomacy happens during turns:
- Proposals can be sent any time
- Responses required within turn timer
- Unresponded proposals expire
- Critical decisions (war) pause timer briefly

## Anti-Griefing Measures

### Vote Kick

Players can vote to remove a player:
- Requires unanimous (minus target)
- Target replaced by AI (or eliminated)
- Used for inactive or trolling players

### Abandonment

If player disconnects:
- 5 turn grace period
- Then AI takes over
- Original player can rejoin

### Collusion Detection

Suspicious patterns flagged:
- Repeated one-sided trades
- Obvious kingmaking
- Coordinated griefing

Not auto-enforced, but logged for review.
