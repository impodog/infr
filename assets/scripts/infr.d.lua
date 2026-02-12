---@meta infr
---@diagnostic disable:undefined-doc-param

--- Defines interface to the infr game.
---@module 'infr'
infr = {}

--- Placeholder for empty/unused ids.
---@global infr.UNUSED_ID integer
infr.UNUSED_ID = 0xfeedd095

--- List of all four directions.
---@global DIRECTIONS Direction[]
infr.DIRECTIONS = { "Right", "Up", "Left", "Down" }

---@alias infr.Direction "Right" | "Up" | "Left" | "Down"

--- Represents a pair of coordinates in the game grid.
---@class infr.Coord
---@field x integer Horizontal coordinate from left to right.
---@field y integer Vertical coordinate from bottom to top.
infr.Coord = {}

--- The type of the object depending on whether and how they make sentences.
--- Instance: The object is only affected, and does not make a sentence.
--- Symbol: Both Instance and Structure.
--- Structure: The object can only be used in sentences.
--- Operator: Special words that only join sentences.
---@alias infr.ObjectKind "Instance" | "Symbol" | "Structure" | "Operator"

--- A minimal description of a game object transferred between Rust and Lua.
---@class infr.ObjectDesc
---@field id integer Globally unique identifier of the object.
---@field direction infr.Direction | nil The direction the object is facing. This may be nil for no specific direction.
---@field kind infr.ObjectKind The type of the object.
---@field group string[] When lua receives, this is all the groups the object is proven to be in; When lua sends, this may only have one item, describing the object's nature.
infr.ObjectDesc = {}

--- The manner of movement. This decides many behaviors and visual effects.
--- Teleport: The object is directly moved to a new position assuming there are no side effects.
--- Remove: Removes the object.
--- Swipe: Moves the object in a direction, and may cause side effects such as pushing.
--- Add: Adds a new object. Note that the object description should use UNUSED_ID, and an id will be assigned by Rust later.
---@alias infr.Manner {kind: "Teleport" | "Remove" | "Placeholder"} | {kind: "Swipe", direction: infr.Direction} | {kind: "Add", object: infr.ObjectDesc}

--- A single movement of a object. This controls the whole game's actions and follows strict logical rules.
--- Please note that in one round, one object can only have one movement(ignoring duplicates). Otherwise it is a logical failure.
---@class infr.Movement
---@field manner infr.Manner The manner of of the movement.
---@field object integer The id of the object that performs this movement.
---@field dest infr.Coord The destination.
---@field prereqs integer[] | nil Ids of objects whose movement must be performed before this movement can be performed.
---@field postreqs integer[] | nil Ids of objects whose movement must be performed after this movement.
---@field disables integer[] If this movement is performed, all the specified movements will not be performed.
--- If you want the movement to be always disabled, set object to `infr.UNUSED_ID`.
infr.Movement = {}

--- The signal sent by stepping the layout. Each signal corresponds to a "round", where one or many "round"s happen after one player input.
--- That is to say, only the first "round" have a direction(player input), and subsequent rounds don't(direction is nil).
---@class infr.Signal
---@field direction infr.Direction | nil
---@field round integer
infr.Signal = {}

--- A compact version of all objects, where you can access objects in certain coordinates.
---@alias infr.Layout table<infr.Coord, infr.ObjectDesc[]>

--- A function that is executed when the layout is stepped (at the start of each round).
---@alias infr.OnInput fun(layout: infr.Layout, signal: infr.Signal, object: integer, coord: infr.Coord): infr.Movement[]

--- Returned by get_listen, instructing the feature to listen for specific movements, for optimization.
---@alias infr.ListenKind infr.Coord | integer

--- A function that is executed after "on_input", determining which objects they should listen to.
--- Only listened objects' movements will be sent to "respond".
---@alias infr.GetListen fun(layout: infr.Layout, object: integer, coord: infr.Coord): infr.ListenKind[]

--- Responds to listened objects' movements.
---@alias infr.Respond fun(layout: infr.Layout, movement: infr.Movement, object: integer, coord: infr.Coord): infr.Movement[]

--- A collection of functions that will be applied to object that are proved in this feature group.
---@class infr.Feature
---@field name string The name of the feature. It should be CamelCase for good practice.
---@field on_input infr.OnInput | nil Executed at the start of a round.
---@field get_listen infr.GetListen | nil Executed after "on_input", determines what movements to "respond" to.
---@field respond infr.Respond | nil Executed for each listened movement.
infr.Feature = {}

--- Registers a feature, returning its id.
---@param feature infr.Feature
---@return integer feature_id The assigned id of the feature.
function infr.register_feature(feature) end

--- Applies for a table local to the current player input sequence and unique to each object.
--- This can be useful for tracking custom information.
---@param object integer The object that this table tracks.
---@return table table The grabbed table.
function infr.grab_table(object) end

--- Moves the coordinates one tile in the given direction.
---@param coord infr.Coord
---@param direction infr.Direction
---@return infr.Coord
function infr.Coord.move(coord, direction) end
