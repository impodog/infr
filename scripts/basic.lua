register_feature({
    name = "You",
    on_input = function(layout, signal, object, coord)
        for key, value in pairs(signal) do
            print(key, "=", value)
        end
        if signal.direction ~= nil then
            local dest = Coord.move(coord, signal.direction)
            print("dest is", dest)
            return { {
                manner = { kind = "Swipe", direction = signal.direction },
                object = object,
                required_by = UNUSED_ID,
                dest = dest,
                forbid = false,
            }, }
        else
            return {}
        end
        return
    end,
})
