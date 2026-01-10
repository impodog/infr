---@meta infr
---@diagnostic disable:undefined-doc-param

--- Placeholder for empty/unused ids.
---@global UNUSED_ID integer
UNUSED_ID = 0xfeedd095

--- List of all four directions.
---@global DIRECTIONS Direction[]
DIRECTIONS = { "Right", "Up", "Left", "Down" }

---@alias Direction "Right" | "Up" | "Left" | "Down"

--- Represents a pair of coordinates in the game grid.
---@class Coord
---@field x integer Horizontal coordinate from left to right.
---@field y integer Vertical coordinate from bottom to top.
Coord = {}

--- The type of the object depending on whether and how they make sentences.
--- Instance: The object is only affected, and does not make a sentence.
--- Symbol: Both Instance and Structure.
--- Structure: The object can only be used in sentences.
--- Operator: Special words that only join sentences.
---@alias ObjectKind "Instance" | "Symbol" | "Structure" | "Operator"

--- A minimal description of a game object transferred between Rust and Lua.
---@class ObjectDesc
---@field id integer Globally unique identifier of the object.
---@field direction Direction | nil The direction the object is facing. This may be nil for no specific direction.
---@field kind ObjectKind The type of the object.
---@field group string[] When lua receives, this is all the groups the object is proven to be in; When lua sends, this may only have one item, describing the object's nature.
ObjectDesc = {}

--- The manner of movement. This decides many behaviors and visual effects.
--- Teleport: The object is directly moved to a new position assuming there are no side effects.
--- Remove: Removes the object.
--- Swipe: Moves the object in a direction, and may cause side effects such as pushing.
--- Add: Adds a new object. Note that the object description should use UNUSED_ID, and an id will be assigned by Rust later.
---@alias Manner {kind: "Teleport" | "Remove"} | {kind: "Swipe", direction: Direction} | {kind: "Add", object: ObjectDesc}

--- A single movement of a object. This controls the whole game's actions and follows strict logical rules.
--- Please note that in one round, one object can only have one movement(ignoring duplicates). Otherwise it is a logical failure.
---@class Movement
---@field manner Manner The manner of of the movement.
---@field object integer The id of the object that performs this movement.
---@field dest Coord The destination.
---@field required_by integer If not UNUSED_ID, this is the id of the object whose movement can only be performed if this movement is performed.
---@field forbid boolean If set to true, this movement can never be performed.
Movement = {}

--- The signal sent by stepping the layout. Each signal corresponds to a "round", where one or many "round"s happen after one player input.
--- That is to say, only the first "round" have a direction(player input), and subsequent rounds don't(direction is nil).
---@class Signal
---@field direction Direction | nil
---@field round integer
Signal = {}

--- A compact version of all objects, where you can access objects in certain coordinates.
---@alias Layout table<Coord, ObjectDesc[]>

--- A function that is executed when the layout is stepped (at the start of each round).
---@alias OnInput fun(layout: Layout, signal: Signal, object: integer, coord: Coord): Movement[]

--- A function that is executed after "on_input", determining which objects they should listen to.
--- Only listened objects' movements will be sent to "respond".
---@alias GetListen fun(layout: Layout, object: integer, coord: Coord): integer[]

--- Responds to listened objects' movements.
---@alias Respond fun(layout: Layout, movement: Movement, object: integer, coord: Coord): Movement[]

--- A collection of functions that will be applied to object that are proved in this feature group.
---@class Feature
---@field name string The name of the feature. It should be CamelCase for good practice.
---@field on_input OnInput | nil Executed at the start of a round.
---@field get_listen GetListen | nil Executed after "on_input", determines what movements to "respond" to.
---@field respond Respond | nil Executed for each listened movement.
Feature = {}

--- Creates an instance type that are tied to a feature, and are objects that have illustrations(in the frontend).
---@class Instance
---@field feature integer The feature that this instance is tied to.
Instance = {}

--- Registers a feature, returning its id.
---@param feature Feature
---@return integer The assigned id of the feature.
function register_feature(feature) end

--- Registers an instance, returning its id. Instances are tied to only one feature.
---@param instance Instance
---@return integer The assigned id of the instance.
function register_instance(instance) end

--- Applies for a table local to the current player input sequence and unique to each object.
--- This can be useful for tracking custom information.
---@param object integer The object that this table tracks.
---@return table The grabbed table.
function grab_table(object) end

--- Moves the coordinates one tile in the given direction.
---@param coord Coord
---@param direction Direction
---@return Coord
function Coord.move(coord, direction) end
