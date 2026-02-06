infr.register_feature({
    name = "You",
    on_input = function(layout, signal, object, coord)
        if signal.direction ~= nil then
            local dest = infr.Coord.move(coord, signal.direction)
            return { {
                manner = { kind = "Swipe", direction = signal.direction },
                object = object,
                dest = dest,
                forbid = false,
            }, }
        else
            return {}
        end
    end,
})
