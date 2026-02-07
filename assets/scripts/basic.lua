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
