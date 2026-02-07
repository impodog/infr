infr.register_feature({
    name = "You",
    on_input = function(layout, signal, object, coord)
        if signal.direction ~= nil then
            local dest = infr.Coord.move(coord, signal.direction)
            return { {
                manner = { kind = "Swipe", direction = signal.direction },
                object = object,
                dest = dest,
            }, }
        else
            return {}
        end
    end,
})

infr.register_feature({
    name = "Stop",
    get_listen = function(layout, object, coord)
        return { coord }
    end,
    respond = function(layout, movement, object, coord)
        return { {
            manner = "Placeholder", -- Manner doesn't matter since this object is not moving anyways
            object = object,
            dest = coord,
            disables = { movement.object }
        } }
    end,
})

infr.register_feature({
    name = "Push",
    get_listen = function(layout, object, coord)
        return { coord }
    end,
    respond = function(layout, movement, object, coord)
        if movement.manner.kind == "Swipe" then
            local direction = movement.manner.direction
            local dest = infr.Coord.move(coord, direction);
            return { {
                manner = { kind = "Swipe", direction = direction },
                object = object,
                dest = dest,
                prereqs = { movement.object },
                postreqs = { movement.object },
            } }
        else
            return {}
        end
    end,
})
