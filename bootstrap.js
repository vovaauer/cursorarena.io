import init, { Game } from './pkg/cursorarena_io.js';

// Multiplayer client (dumb renderer)
async function initMultiplayerGame(mainContent, mapData) {
    const socket = new WebSocket('wss://cursorarena.vovaauer.com:8088');
    let localPlayerId = null;

    const canvas = mainContent.querySelector('canvas');
    const ctx = canvas.getContext('2d');

    const cursorDefault = new Image();
    cursorDefault.src = 'assets/default_arrow.png';
    const cursorGrabbing = new Image();
    cursorGrabbing.src = 'assets/default_link.png';

    const WORLD_WIDTH = 16.0;
    const WORLD_HEIGHT = 9.0;
    const WORLD_ASPECT_RATIO = WORLD_WIDTH / WORLD_HEIGHT;
    const CURSOR_DRAW_SIZE = 0.35;
    let scale = 1.0;
    const inputState = {
        mouse_dx: 0,
        mouse_dy: 0,
        isMouseDown: false,
    };

    socket.onopen = function(event) {
        console.log('[open] Connection established');
        document.addEventListener("mousemove", updatePosition, false);
        window.addEventListener('mousedown', () => { inputState.isMouseDown = true; });
        window.addEventListener('mouseup', () => { inputState.isMouseDown = false; });
        setInterval(sendInput, 1000 / 60);
    };

    socket.onmessage = function(event) {
        try {
            const message = JSON.parse(event.data);
            if (message.type === 'Welcome') {
                localPlayerId = message.id;
            } else if (message.type === 'GameState') {
                draw(message);
            }
        } catch (e) {
            console.error('Error parsing message:', e);
        }
    };

    socket.onclose = function(event) {
        document.removeEventListener("mousemove", updatePosition, false);
    };

    function handleResize() {
        const rect = mainContent.getBoundingClientRect();
        canvas.width = rect.width;
        canvas.height = rect.height;
        const canvasAspectRatio = canvas.width / canvas.height;
        if (canvasAspectRatio > WORLD_ASPECT_RATIO) {
            scale = canvas.height / WORLD_HEIGHT;
        } else {
            scale = canvas.width / WORLD_WIDTH;
        }
    }
    window.addEventListener('resize', handleResize);
    handleResize();

    canvas.addEventListener('click', () => {
        canvas.requestPointerLock();
    });

    function updatePosition(e) {
        inputState.mouse_dx += e.movementX;
        inputState.mouse_dy -= e.movementY;
    }
    
    function sendInput() {
        if (socket.readyState === WebSocket.OPEN) {
            const world_dx = inputState.mouse_dx / scale;
            const world_dy = inputState.mouse_dy / scale;
            const message = {
                mouse_dx: world_dx,
                mouse_dy: world_dy,
                is_mouse_down: inputState.isMouseDown,
            };
            socket.send(JSON.stringify(message));
            inputState.mouse_dx = 0;
            inputState.mouse_dy = 0;
        }
    }

    function draw(gameState) {
        ctx.fillStyle = '#222';
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        ctx.save();
        ctx.translate(canvas.width / 2, canvas.height / 2);
        ctx.scale(scale, -scale);

        ctx.fillStyle = 'white';
        gameState.boundaries.forEach(b => {
            ctx.fillRect(b.x - b.half_width, b.y - b.half_height, b.half_width * 2, b.half_height * 2);
        });

        gameState.objects.forEach(obj => {
            ctx.save();
            ctx.translate(obj.x, obj.y);
            ctx.rotate(obj.rotation);
            switch (obj.user_data) {
                case 1: ctx.fillStyle = '#3498db'; break;
                case 2: ctx.fillStyle = '#e74c3c'; break;
                default: ctx.fillStyle = 'white'; break;
            }
            switch (obj.shape) {
                case 'Square': ctx.fillRect(-obj.half_width, -obj.half_height, obj.half_width * 2, obj.half_height * 2); break;
                case 'Circle': ctx.beginPath(); ctx.arc(0, 0, obj.radius, 0, Math.PI * 2); ctx.fill(); break;
            }
            ctx.restore();
        });

        gameState.players.forEach(player => {
            const isLocalPlayer = player.id === localPlayerId;
            drawCursor(player.x, player.y, player.is_grabbing, player.is_over_grabbable, isLocalPlayer);
        });

        ctx.restore();
    }

    function drawCursor(x, y, isGrabbing, isOverGrabbable, isLocalPlayer) {
        const cursorImg = (isGrabbing || isOverGrabbable) ? cursorGrabbing : cursorDefault;
        if (cursorImg.complete && cursorImg.naturalWidth !== 0) {
            const cursorAspect = cursorImg.naturalWidth / cursorImg.naturalHeight;
            const cursorHeight = CURSOR_DRAW_SIZE / cursorAspect;
            ctx.save();
            ctx.translate(x, y);
            ctx.scale(1, -1);
            if (!isLocalPlayer) {
                ctx.globalAlpha = 0.5;
            }
            ctx.drawImage(cursorImg, 0, 0, CURSOR_DRAW_SIZE, cursorHeight);
            ctx.restore();
        }
    }
}

// Local game simulation for map editor
async function initLocalGame(mainContent, mapData) {
    await init();

    const canvas = mainContent.querySelector('canvas');
    const ctx = canvas.getContext('2d');

    const cursorDefault = new Image();
    cursorDefault.src = 'assets/default_arrow.png';
    const cursorGrabbing = new Image();
    cursorGrabbing.src = 'assets/default_link.png';

    const game = new Game(mapData || null);

    const WORLD_WIDTH = 16.0;
    const WORLD_HEIGHT = 9.0;
    const WORLD_ASPECT_RATIO = WORLD_WIDTH / WORLD_HEIGHT;
    const CURSOR_DRAW_SIZE = 0.35;
    let scale = 1.0;
    const inputState = {
        mouse_dx: 0,
        mouse_dy: 0,
        isMouseDown: false,
    };

    function handleResize() {
        const rect = mainContent.getBoundingClientRect();
        canvas.width = rect.width;
        canvas.height = rect.height;
        const canvasAspectRatio = canvas.width / canvas.height;
        if (canvasAspectRatio > WORLD_ASPECT_RATIO) {
            scale = canvas.height / WORLD_HEIGHT;
        } else {
            scale = canvas.width / WORLD_WIDTH;
        }
    }
    window.addEventListener('resize', handleResize);
    handleResize();

    canvas.addEventListener('click', () => {
        canvas.requestPointerLock();
    });

    function updatePosition(e) {
        inputState.mouse_dx += e.movementX;
        inputState.mouse_dy -= e.movementY;
    }

    document.addEventListener('pointerlockchange', () => {
        if (document.pointerLockElement === canvas) {
            document.addEventListener("mousemove", updatePosition, false);
        } else {
            document.removeEventListener("mousemove", updatePosition, false);
        }
    }, false);

    window.addEventListener('mousedown', () => { inputState.isMouseDown = true; });
    window.addEventListener('mouseup', () => { inputState.isMouseDown = false; });

    function gameLoop() {
        const world_dx = inputState.mouse_dx / scale;
        const world_dy = inputState.mouse_dy / scale;
        inputState.mouse_dx = 0;
        inputState.mouse_dy = 0;

        game.tick(world_dx, world_dy, inputState.isMouseDown);

        const gameStateString = game.get_game_state();
        const gameState = JSON.parse(gameStateString);

        draw(gameState);
        requestAnimationFrame(gameLoop);
    }

    function draw(gameState) {
        ctx.fillStyle = '#222';
        ctx.fillRect(0, 0, canvas.width, canvas.height);
        ctx.save();
        ctx.translate(canvas.width / 2, canvas.height / 2);
        ctx.scale(scale, -scale);

        ctx.fillStyle = 'white';
        gameState.boundaries.forEach(b => {
            ctx.fillRect(b.x - b.half_width, b.y - b.half_height, b.half_width * 2, b.half_height * 2);
        });

        gameState.objects.forEach(obj => {
            ctx.save();
            ctx.translate(obj.x, obj.y);
            ctx.rotate(obj.rotation);
            switch (obj.user_data) {
                case 1: ctx.fillStyle = '#3498db'; break;
                case 2: ctx.fillStyle = '#e74c3c'; break;
                default: ctx.fillStyle = 'white'; break;
            }
            switch (obj.shape) {
                case 'Square': ctx.fillRect(-obj.half_width, -obj.half_height, obj.half_width * 2, obj.half_height * 2); break;
                case 'Circle': ctx.beginPath(); ctx.arc(0, 0, obj.radius, 0, Math.PI * 2); ctx.fill(); break;
            }
            ctx.restore();
        });

        gameState.players.forEach(player => {
            drawCursor(player.x, player.y, player.is_grabbing, player.is_over_grabbable);
        });

        ctx.restore();
    }

    function drawCursor(x, y, isGrabbing, isOverGrabbable) {
        const cursorImg = (isGrabbing || isOverGrabbable) ? cursorGrabbing : cursorDefault;
        if (cursorImg.complete && cursorImg.naturalWidth !== 0) {
            const cursorAspect = cursorImg.naturalWidth / cursorImg.naturalHeight;
            const cursorHeight = CURSOR_DRAW_SIZE / cursorAspect;
            ctx.save();
            ctx.translate(x, y);
            ctx.scale(1, -1);
            ctx.drawImage(cursorImg, 0, 0, CURSOR_DRAW_SIZE, cursorHeight);
            ctx.restore();
        }
    }

    requestAnimationFrame(gameLoop);
    return game;
}

export { initMultiplayerGame, initLocalGame };
export default initMultiplayerGame;
