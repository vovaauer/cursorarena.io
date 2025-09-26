# Cursor Arena Map Editor API

This document describes the Lua API for creating maps in Cursor Arena.

## Global Functions

These functions control the global properties of the map.

### `set_gravity(x, y)`

Sets the gravity of the map.

-   `x` (number): The x component of the gravity vector.
-   `y` (number): The y component of the gravity vector.

Example:
```lua
set_gravity(0, -9.8)
```

### `set_cursor_size(size)`

Sets the size of the player's cursor.

-   `size` (number): The new size of the cursor.

Example:
```lua
set_cursor_size(0.5)
```

### `set_map_dimensions(width, height)`

Sets the dimensions of the map.

-   `width` (number): The width of the map.
-   `height` (number): The height of the map.

Example:
```lua
set_map_dimensions(16, 9)
```

## Map Objects

Maps are made of objects. You can create different types of objects with different properties.

### `create_entity(properties)`

Creates a new object in the map.

-   `properties` (table): A table of properties for the object.

**Properties:**

-   `shape` (string): The shape of the object. Can be `"rect"` or `"circle"`.
-   `x1`, `y1`, `x2`, `y2` (numbers): The coordinates for a `"rect"` shape, in the range 0-1.
-   `x`, `y`, `radius` (numbers): The coordinates and radius for a `"circle"` shape.
-   `is_static` (boolean): If `true`, the object will not move. Default is `false`.
-   `is_death` (boolean): If `true`, the object will kill players on contact. Default is `false`.
-   `restitution` (number): The bounciness of the object. Default is `0.0`.
-   `parent` (object): Another object to be the parent of this object. (Not yet implemented)

**Returns:**

The created object.

Example:
```lua
local wall = create_entity({
  shape = "rect",
  x1 = 0, y1 = 0, x2 = 1, y2 = 0.1,
  is_static = true
})

local bouncy_ball = create_entity({
  shape = "circle",
  x = 0.5, y = 0.5, radius = 0.1,
  restitution = 0.8
})
```

## Events

You can respond to game events by defining functions.

### `on_mouse_click(callback)`

Sets a function to be called when the mouse is clicked in the map preview.

-   `callback` (function): A function that takes `x` and `y` as arguments.

Example:
```lua
on_mouse_click(function(x, y)
  print("Mouse clicked at: " .. x .. ", " .. y)
end)
```

### `on_object_collision(callback)`

Sets a function to be called when two objects collide.

-   `callback` (function): A function that takes `object1` and `object2` as arguments.

Example:
```lua
on_object_collision(function(object1, object2)
  print("Collision detected!")
end)
```
