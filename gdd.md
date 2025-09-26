## Game Design Document: `cursorarena.io`

**Version:** 1.0
**Date:** September 25, 2025
**Author:** (Your Name/Studio Name)

*This is a living document. It is expected to evolve throughout the development process as new ideas are prototyped and tested. All changes should be versioned and documented.*

---

### 1. Vision & Core Concept

#### 1.1. Logline

A skill-based, physics-driven arena brawler where your cursor is your weapon, your shield, and your life. Master the art of the fling to survive in a minimalist world of user-generated chaos.

#### 1.2. Game Summary

`cursorarena.io` is a 2D, free-for-all, last-man-standing game playable in a web browser. Players control their actual mouse cursor in a single-screen arena. The objective is to be the last survivor by using a simple yet profound physics system to grab environmental objects and launch them at opponents. With an emphasis on pure mechanical skill and a powerful integrated map editor, `cursorarena.io` is designed to be an infinitely replayable, community-driven competitive experience.

#### 1.3. Genre

FFA Arena Brawler, Physics Party Game, .io Game

#### 1.4. Target Audience

Fans of skill-based competitive games with a high ceiling for mastery, physics sandbox games, and accessible browser-based (.io) games.
*   **Primary Inspirations:** *Agar.io*, *Stick Fight: The Game*, *N++*, *Ultimate Chicken Horse*.

#### 1.5. Design Pillars

These four pillars are the fundamental principles that guide every design and development decision.

1.  **Pure Skill Expression:** The winner is determined solely by player dexterity, strategy, and understanding of the physics. There will be no random elements (e.g., random crits, random item spawns) that influence the outcome of a direct engagement. Control must be 1:1 and instantly responsive.
2.  **Infinite Canvas (Player Creativity):** The map editor is a first-class citizen, not an add-on. The long-term health and variety of the game will be driven by user-generated content. The tools provided must be simple to use but deep enough to allow for complex and novel creations.
3.  **Immediate Readability:** The visual language must be minimalist and unambiguous. A new player should understand the function of every object on screen within seconds of joining a match. Form must always follow function.
4.  **Architected for Scale:** The game is being built from day one with a client-server architecture in mind. The strict separation of game logic (Rust/WASM) from rendering (JS/Canvas) ensures performance, security, and a clear path to a robust, authoritative multiplayer experience.

---

### 2. Gameplay Mechanics

#### 2.1. Core Loop

The central gameplay loop is designed to be fast, addictive, and easy to learn.

1.  **Join:** The player enters a match from the main menu.
2.  **Maneuver & Assess:** The player moves their cursor, scanning the arena for threats (other players, Death objects) and opportunities (Grabbable objects).
3.  **Grab & Fling:** The player grabs an object, uses their mouse movement to build momentum, and releases it at a strategic moment to attack or create a barrier.
4.  **Eliminate & Survive:** Players are eliminated on contact with Death objects. The primary goal is to use the environment to cause other players to be eliminated.
5.  **Repeat:** The loop continues until only one player remains.

#### 2.2. Player Controls

*   **Mouse Movement:** Directly controls the position of the player's cursor on screen.
*   **Left Mouse Button (Hold):** Initiates a "grab." The nearest `Grabbable` object within a small radius is physically tethered to the cursor's position.
*   **Left Mouse Button (Release):** Releases the grabbed object, imparting the cursor's current velocity onto it. This is the core "fling" mechanic.

#### 2.3. Physics World & Objects

The world is composed of four fundamental object types. All objects (except the cursor) can have their properties (e.g., static, dynamic) set in the map editor.

*   **Cursor (Player):** The player's avatar. Has no physical body itself but is the anchor point for grabbing. If the center of the cursor touches a `Death` object, the player is eliminated.
*   **Wall:** Impassable terrain for cursors. `Dynamic` walls can be pushed by other physics objects.
*   **Grabbable:** The primary tool and ammunition of the game. Can be passed through by cursors but collides with other objects. This is the only object type that can be directly manipulated by the player.
*   **Death:** The primary hazard. Any cursor touching a `Death` object is instantly eliminated. `Dynamic` Death objects are the most potent weapons in the game.

#### 2.4. Parent/Child System

This is a key feature for advanced map creation. In the map editor, any two objects can be "parented" together.
*   **Function:** Parented objects act as a single, rigid physics body, even if they are not touching.
*   **Emergent Gameplay:** This allows for the creation of complex contraptions like flails (a `Grabbable` handle parented to a distant `Death` block), rotating platforms, or complex barriers.

#### 2.5. Game Modes

*   **Last Man Standing (LMS):** The default FFA mode. The last surviving player wins the round.
*   **Control Point:** A designated zone appears on the map. A player must remain within the zone for a set duration to win. This forces confrontation and creates a focal point for the action, preventing overly passive play.

---

### 3. The Map Editor

The map editor is the engine of community engagement and long-term replayability.

#### 3.1. Vision

To provide a set of simple, intuitive tools that allow players to create, test, and share their own unique arenas and challenges with the world.

#### 3.2. Core Features

*   **Object Palette:** Simple UI to select Wall, Grabbable, or Death objects.
*   **Transform Tools:** Place, move, rotate, and scale any object.
*   **Properties Panel:** Select an object to edit its properties (e.g., Static/Dynamic, Color).
*   **Parenting Tool:** A simple "click-to-link" tool to parent two objects.
*   **Map Settings:** Configure global map rules:
    *   Gravity (vector direction and magnitude).
    *   Player spawn points.
    *   Default game mode.
    *   Boundary/aspect ratio.
    *   Background color.
*   **Community Integration:** Save, upload, and share maps. A browser for searching and rating community-made maps.

---

### 4. Art, Audio, and UI/UX

#### 4.1. Art Direction

The aesthetic is **Functional Minimalism**. Visuals must be clean, high-contrast, and instantly readable.
*   **Shapes:** Geometric primitives (circles, squares, simple polygons).
*   **Color Palette:** Limited and functional. For example, all `Death` objects will be a consistent color (e.g., bright red) to be instantly recognizable. The background and other objects will use a clean, high-contrast palette.
*   **Effects:** Simple particle effects for collisions, player elimination, and cosmetic trails. The focus is on game feel, not visual noise.

#### 4.2. Audio Direction

Audio provides critical feedback and enhances the "feel" of the physics.
*   **Sound Effects (SFX):** Distinct, satisfying sounds for: grab, fling/release, object-wall collision, object-object collision, and a definitive player elimination sound.
*   **Music:** Minimalist, atmospheric electronic music that builds in intensity as the number of remaining players decreases.

#### 4.3. UI/UX

The UI should be clean, unobtrusive, and fast.
*   **Main Menu:** Large, clear buttons for [Play], [Map Editor], [Shop], [Settings].
*   **In-Game HUD:** Extremely minimal. The only persistent UI element will be a player count in a corner of the screen. All other information is conveyed through the game world itself.
*   **Post-Game Screen:** Displays the winner and offers options to play again, return to the menu, or rate the map.

---

### 5. Technical Specification

#### 5.1. Platform

*   **Primary:** Web Browser (PC), leveraging WebAssembly for high-performance logic.
*   **Secondary:** Potential for a standalone desktop client (via Electron or similar) for dedicated players seeking minimal latency.

#### 5.2. Core Architecture

*   **Client-Server Model:** A strict separation between client and server logic.
*   **Game Logic (Rust -> WASM):** An authoritative "headless" instance of the game. It will manage the entire `rapier2d` physics simulation, process all player inputs, and hold the true game state.
*   **Client (JavaScript / HTML5 Canvas):** A "dumb" client. Its sole responsibilities are capturing and sending user input to the server/logic module, and rendering the game state it receives back.
*   **Networking Protocol:** WebSockets for low-latency, real-time, bi-directional communication.
*   **Player Count:** Initial target of 8-16 players per lobby.

---

### 6. Monetization & Community

#### 6.1. Business Model

Free-to-Play. The core game experience will be free for everyone.

#### 6.2. Monetization Strategy

Monetization will be **100% cosmetic** and will never offer a competitive advantage.
*   **Custom Cursor Skins:** Visual variations of the player cursor.
*   **Cursor Trails:** Particle effects that follow the cursor's movement.
*   **Nametag Customization:** Custom colors and flair for the player's nameplate.
*   **Creator Support:** Potential for a revenue-sharing model where players can tip or support creators of popular maps.