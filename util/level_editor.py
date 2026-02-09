import argparse
import json
import os
import time
from copy import deepcopy

import pygame
import toml


def add_vec(v1, v2):
    return (v1[0] + v2[0], v1[1] + v2[1])


def add_rect(r1, v2):
    return (r1[0] + v2[0], r1[1] + v2[1], r1[2] + v2[0], r1[3] + v2[1])


def sub_vec(v1, v2):
    return (v1[0] - v2[0], v1[1] - v2[1])


def mul_vec(v, k):
    return (v[0] * k, v[1] * k)


def new_object(coord) -> dict:
    return {"direction": 0, "group": "$Empty", "coord": coord, "flags": []}


FRAMERATE = 60.0
SPRITES = {
    "Empty": "empty/empty.png",
    "$Box": "box/box.png",
    "$Omega": "omega/omega.png",
    "$Wall": "wall/wall.png",
}
SCREEN_SIZE = (1920, 1080)
EDITOR_SIZE = (1500, 1080)
STANDARD_OBJECT_SIZE = 32
DEFAULT_OBJECT_SIZE = 64
ALIGN_LINE_COLOR = (100, 100, 100)
HOVER_COLOR = (20, 100, 100, 120)
FONT_SIZE = 24
UI_SIZE = sub_vec(SCREEN_SIZE, EDITOR_SIZE)
UI_DISTANCE = 30
UI_SELECTED_COLOR = (100, 255, 100)
UI_UNSELECTED_COLOR = (255, 255, 255)
UI_INFO_COLOR = (200, 200, 200)

pygame.init()
screen = pygame.display.set_mode(SCREEN_SIZE)

loaded_sprites = dict()
default_sprite = pygame.image.load("assets/sprites/" + SPRITES["Empty"]).convert()
font = pygame.font.Font("assets/fonts/JetbrainsMono.ttf", FONT_SIZE)


def decide_sprite(nature: str) -> pygame.Surface:
    global loaded_sprites
    if nature not in SPRITES:
        nature = nature[1:]
    if sprite := loaded_sprites.get(nature):
        return sprite
    else:
        if path := SPRITES.get(nature):
            path = "assets/sprites/" + path
            sprite = pygame.image.load(path).convert()
            sprite.set_clip((0, 0, 32, 32))
            loaded_sprites[nature] = sprite
            return sprite
        else:
            return default_sprite


def main():
    def test_ui_clicks() -> bool:
        nonlocal mouse_pos, ui_selected, current_pos, clicked, current_ui
        # print(f"REQUIRE {current_pos[1]} found {mouse_pos[1]}")
        if (
            clicked
            and EDITOR_SIZE[0] < mouse_pos[0] < SCREEN_SIZE[0]
            and current_pos[1] < mouse_pos[1] < current_pos[1] + UI_DISTANCE
        ):
            ui_selected = current_ui
        return ui_selected == current_ui

    def next_ui():
        nonlocal current_pos, current_ui
        current_pos = (current_pos[0], current_pos[1] + UI_DISTANCE)
        current_ui += 1

    def coord_to_rect(coord):
        nonlocal displacement, object_size
        center = mul_vec(coord, object_size)
        return pygame.Rect(
            (
                center[0] - object_size / 2 + displacement[0],
                -center[1] - object_size / 2 + displacement[1],
            ),
            (object_size, object_size),
        )

    def input_on(s: str) -> str:
        nonlocal keys
        for key, unicode in keys:
            if key == pygame.K_BACKSPACE:
                s = s[:-1]
            else:
                s = s + unicode
        return s

    def show_selectable_ui(s: str):
        nonlocal current_pos
        screen.blit(
            font.render(
                s,
                True,
                UI_SELECTED_COLOR if test_ui_clicks() else UI_UNSELECTED_COLOR,
            ),
            current_pos,
        )

    def equal_coord(x, y):
        return x[0] == y[0] and x[1] == y[1]

    pygame.init()

    parser = argparse.ArgumentParser()
    parser.add_argument("file", type=str)
    args = parser.parse_args()

    object_size = DEFAULT_OBJECT_SIZE
    displacement = mul_vec(EDITOR_SIZE, 0.5)

    if not os.path.exists(args.file):
        data = {
            "requirements": [],
            "meta": {"title": "INPUT_TITLE_HERE"},
            "objects": [],
        }
    else:
        with open(args.file, "r") as file:
            data = json.load(file)

    for object in data["objects"]:
        if object.get("flags") is None:
            object["flags"] = list()

    pygame.display.set_caption(f"Edit: {args.file}")

    loop = True
    selected = None
    dragged = None
    selection_pos = None
    ui_selected = None
    pressed = set()
    while loop:
        start_time = time.time()
        screen.fill((0, 0, 0))

        hover_coord = None
        mouse_pos = pygame.mouse.get_pos()
        clicked = False
        right_clicked = False
        keys = list()
        if 0 <= mouse_pos[0] < EDITOR_SIZE[0] and 0 <= mouse_pos[1] < EDITOR_SIZE[1]:
            x = int(round((mouse_pos[0] - displacement[0]) / object_size))
            y = int(round((-mouse_pos[1] + displacement[1]) / object_size))
            hover_coord = (x, y)

        # Listen to input
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                loop = False
            elif event.type == pygame.MOUSEBUTTONDOWN:
                if event.button == pygame.BUTTON_LEFT:
                    clicked = True
                    if hover_coord:
                        in_here = list()
                        for object in data["objects"]:
                            if (
                                object["coord"][0] == hover_coord[0]
                                and object["coord"][1] == hover_coord[1]
                            ):
                                in_here.append(object)
                        if len(in_here) > 0:
                            if selected is None:
                                selected = in_here[0]
                            else:
                                any_present = False
                                for index in range(len(in_here)):
                                    if in_here[index] == selected:
                                        selected = in_here[(index + 1) % len(in_here)]
                                        any_present = True
                                if not any_present:
                                    selected = in_here[0]
                            ui_selected = None
                            dragged = selected
                elif event.button == pygame.BUTTON_RIGHT:
                    right_clicked = True
            elif event.type == pygame.MOUSEWHEEL:
                object_size += event.y * 10
                object_size = max(STANDARD_OBJECT_SIZE, object_size)
            elif event.type == pygame.MOUSEBUTTONUP:
                if event.button == pygame.BUTTON_LEFT:
                    if dragged:
                        dragged["coord"] = hover_coord
                        dragged = None
            elif event.type == pygame.KEYDOWN:
                keys.append((event.key, event.unicode))
                pressed.add(event.key)
                if event.key == pygame.K_ESCAPE:
                    selected = dragged = ui_selected = None
            elif event.type == pygame.KEYUP:
                pressed.remove(event.key)

        ctrl = pygame.K_LCTRL in pressed or pygame.K_RCTRL in pressed
        shift = pygame.K_LSHIFT in pressed or pygame.K_RSHIFT in pressed

        # When right click on an object, it is deleted.
        if right_clicked and selected and equal_coord(hover_coord, selected["coord"]):
            count = 0
            for object in data["objects"]:
                if object == selected:
                    del data["objects"][count]
                    selected = None
                    break
                count += 1

        # When dragging with right button + CTRL, anything that passes through is deleted.
        if pygame.mouse.get_pressed()[2] and hover_coord and ctrl:
            dels = list()
            count = 0
            for object in data["objects"]:
                if equal_coord(object["coord"], hover_coord):
                    dels.append(count)
                count += 1
            dels.reverse()
            for index in dels:
                del data["objects"][index]

        # When clicking / dragging on nothing, objects are added
        if pygame.mouse.get_pressed()[0] and hover_coord and ctrl:
            any_already = False
            for object in data["objects"]:
                if equal_coord(object["coord"], hover_coord):
                    any_already = True
            if not any_already:
                if selected:
                    object = deepcopy(selected)
                    object["coord"] = hover_coord
                    data["objects"].append(object)
                else:
                    data["objects"].append(new_object(hover_coord))

        # Draw alignment lines
        x_base = displacement[0] + object_size / 2
        y_base = displacement[1] + object_size / 2
        x = x_base
        while x < EDITOR_SIZE[0]:
            pygame.draw.line(screen, ALIGN_LINE_COLOR, (x, 0), (x, EDITOR_SIZE[1]))
            x += object_size
        x = x_base
        while x > 0:
            pygame.draw.line(screen, ALIGN_LINE_COLOR, (x, 0), (x, EDITOR_SIZE[1]))
            x -= object_size
        y = y_base
        while y < EDITOR_SIZE[1]:
            pygame.draw.line(screen, ALIGN_LINE_COLOR, (0, y), (EDITOR_SIZE[0], y))
            y += object_size
        y = y_base
        while y > 0:
            pygame.draw.line(screen, ALIGN_LINE_COLOR, (0, y), (EDITOR_SIZE[0], y))
            y -= object_size

        # Blit objects
        for object in data["objects"]:
            rect = coord_to_rect(object["coord"])
            sprite = decide_sprite(object["group"])
            scaled = pygame.transform.scale(sprite, (object_size, object_size))
            screen.blit(scaled, rect)

        # Draw hover indicator
        if hover_coord:
            rect = coord_to_rect(hover_coord)
            surface = pygame.Surface((object_size, object_size), pygame.SRCALPHA)
            surface.fill(HOVER_COLOR)
            screen.blit(surface, rect)

        # Draw the ui as needed
        pygame.draw.line(
            screen,
            (255, 255, 255),
            (EDITOR_SIZE[0], 0),
            (EDITOR_SIZE[0], EDITOR_SIZE[1]),
            3,
        )

        current_pos = (EDITOR_SIZE[0] + 10, 20)
        current_ui = 0
        if selected:
            screen.blit(
                font.render(f"Coord = {selected['coord']}", True, UI_INFO_COLOR),
                current_pos,
            )
            next_ui()
            next_ui()
            show_selectable_ui(f"Group = {selected['group']}")
            if ui_selected == current_ui:
                selected["group"] = input_on(selected["group"])
            next_ui()
            next_ui()
            count = 0
            for flag in selected["flags"]:
                show_selectable_ui(f"Flag {count} = {flag}")
                if ui_selected == current_ui:
                    if right_clicked:
                        del selected["flags"][count]
                        ui_selected = None
                    else:
                        selected["flags"][count] = input_on(flag)
                next_ui()
                count += 1
            show_selectable_ui("[Add flag]")
            if ui_selected == current_ui:
                selected["flags"].append("")
                ui_selected = None
            next_ui()
        else:
            show_selectable_ui(f"Title = {data['meta']['title']}")
            if ui_selected == current_ui:
                data["meta"]["title"] = input_on(data["meta"]["title"])
            next_ui()

        # Post-loop updates and control framerate
        pygame.display.flip()
        end_time = time.time()
        delta_time = end_time - start_time
        if delta_time < 1 / FRAMERATE:
            time.sleep(1 / FRAMERATE - delta_time)

    # Condense the output dict
    for object in data["objects"]:
        if flags := object.get("flags"):
            if len(flags) == 0:
                del object["flags"]

    with open(args.file, "w") as file:
        json.dump(data, file)


if __name__ == "__main__":
    main()
