const { lua, lauxlib, lualib, to_luastring } = window.fengari;

const defaultScript = `-- Welcome to the Cursor Arena Map Editor!
-- This is a default script to get you started.

-- Global map settings
set_gravity(0, -2.0)

-- Create the floor
create_entity({
  shape = "rect",
  x1 = 0.1, y1 = 0.1, x2 = 0.9, y2 = 0.2,
  is_static = true
})

-- Create a bouncy ball
create_entity({
  shape = "circle",
  x = 0.5, y = 0.7, radius = 0.05,
  restitution = 0.8
})

-- Create a death block
create_entity({
  shape = "rect",
  x1 = 0.4, y1 = 0.4, x2 = 0.6, y2 = 0.5,
  is_death = true
})

-- Event handling
on_mouse_click(function(x, y)
  print("Mouse clicked at: " .. x .. ", " .. y)
  -- Create a small square where the mouse was clicked
  create_entity({
    shape = "rect",
    x1 = x - 0.02, y1 = y - 0.02, x2 = x + 0.02, y2 = y + 0.02
  })
end)
`;

let mapData = {};

function api_set_gravity(L) {
    const x = lauxlib.luaL_checknumber(L, 1);
    const y = lauxlib.luaL_checknumber(L, 2);
    mapData.gravity = [x, y];
    return 0;
}

function api_set_map_dimensions(L) {
    const width = lauxlib.luaL_checknumber(L, 1);
    const height = lauxlib.luaL_checknumber(L, 2);
    mapData.dimensions = [width, height];
    return 0;
}

function api_create_entity(L) {
    lauxlib.luaL_checktype(L, 1, lua.LUA_TTABLE);
    const props = {};
    lua.lua_pushnil(L);
    while (lua.lua_next(L, 1) !== 0) {
        const key = lua.lua_tojsstring(L, -2);
        // For now, we just handle numbers and booleans
        if (lua.lua_isnumber(L, -1)) {
            props[key] = lua.lua_tonumber(L, -1);
        } else if (lua.lua_isboolean(L, -1)) {
            props[key] = lua.lua_toboolean(L, -1);
        } else if (lua.lua_isstring(L, -1)) {
            props[key] = lua.lua_tojsstring(L, -1);
        }
        lua.lua_pop(L, 1);
    }
    if (!mapData.entities) {
        mapData.entities = [];
    }
    mapData.entities.push(props);
    // In a real implementation, we would return a userdata representing the entity
    return 0;
}

// Placeholder for event handlers
let onMouseClick;

function api_on_mouse_click(L) {
    lauxlib.luaL_checktype(L, 1, lua.LUA_TFUNCTION);
    onMouseClick = lauxlib.luaL_ref(L, lua.LUA_REGISTRYINDEX);
    return 0;
}



const cursorArenaLib = {
    "set_gravity": api_set_gravity,
    "create_entity": api_create_entity,
    "on_mouse_click": api_on_mouse_click,
    "set_map_dimensions": api_set_map_dimensions,
};

export function runLuaScript(script) {
    mapData = {}; // Reset map data
    const L = lauxlib.luaL_newstate();
    lualib.luaL_openlibs(L);

    lauxlib.luaL_newlib(L, cursorArenaLib);
    lua.lua_setglobal(L, to_luastring("cursor_arena"));

    // For convenience, we can also put our functions in the global namespace
    for (const key in cursorArenaLib) {
        lua.lua_pushcfunction(L, cursorArenaLib[key]);
        lua.lua_setglobal(L, to_luastring(key));
    }

    const result = lauxlib.luaL_dostring(L, to_luastring(script));
    if (result !== lua.LUA_OK) {
        const error = lua.lua_tojsstring(L, -1);
        console.error("Lua error:", error);
        lua.lua_pop(L, 1);
        return null;
    }

    return mapData;
}

export { defaultScript };
