/*
 * screen-res — print the primary display resolution (Wayland-friendly).
 *
 * Usage:
 *   screen-res                 # WIDTHxHEIGHT
 *   screen-res width           # WIDTH
 *   screen-res height          # HEIGHT
 *   screen-res aspect 4:3      # largest WxH that fits the screen at 4:3
 *   screen-res -s ...          # space-separated output (for `read W H`)
 *
 * Wayland note: clients without a focused surface cannot learn which output
 * the cursor is on (SDL_GetGlobalMouseState returns 0,0), so this tool always
 * reports the primary display.
 */

#include <SDL3/SDL.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>

static int usage(const char *prog) {
    fprintf(stderr,
            "Usage: %s [-s|--space] [resolution|width|height|aspect W:H]\n",
            prog);
    return 2;
}

static int parse_aspect(const char *s, int *aw, int *ah) {
    char sep = 0;
    if (sscanf(s, "%d%c%d", aw, &sep, ah) != 3) return 0;
    if (sep != ':' && sep != 'x' && sep != '/') return 0;
    return *aw > 0 && *ah > 0;
}

int main(int argc, char **argv) {
    if (!SDL_Init(SDL_INIT_VIDEO)) {
        fprintf(stderr, "SDL_Init failed: %s\n", SDL_GetError());
        return 1;
    }

    SDL_DisplayID id = SDL_GetPrimaryDisplay();
    if (!id) {
        fprintf(stderr, "No primary display: %s\n", SDL_GetError());
        SDL_Quit();
        return 1;
    }

    const SDL_DisplayMode *mode = SDL_GetCurrentDisplayMode(id);
    if (!mode) {
        fprintf(stderr, "SDL_GetCurrentDisplayMode failed: %s\n", SDL_GetError());
        SDL_Quit();
        return 1;
    }

    float density = mode->pixel_density > 0.0f ? mode->pixel_density : 1.0f;
    int w = (int)(mode->w * density + 0.5f);
    int h = (int)(mode->h * density + 0.5f);

    char sep = 'x';
    const char *positional[3] = {0};
    int npos = 0;
    for (int i = 1; i < argc; i++) {
        if (strcmp(argv[i], "-s") == 0 || strcmp(argv[i], "--space") == 0) {
            sep = ' ';
        } else if (npos < 2) {
            positional[npos++] = argv[i];
        } else {
            SDL_Quit();
            return usage(argv[0]);
        }
    }

    const char *cmd = npos > 0 ? positional[0] : "resolution";
    int rc = 0;

    if (strcmp(cmd, "resolution") == 0) {
        printf("%d%c%d\n", w, sep, h);
    } else if (strcmp(cmd, "width") == 0) {
        printf("%d\n", w);
    } else if (strcmp(cmd, "height") == 0) {
        printf("%d\n", h);
    } else if (strcmp(cmd, "aspect") == 0) {
        if (npos < 2) { rc = usage(argv[0]); goto done; }
        int aw, ah;
        if (!parse_aspect(positional[1], &aw, &ah)) {
            fprintf(stderr, "Invalid aspect ratio %s (expected e.g. 4:3)\n", positional[1]);
            rc = 2; goto done;
        }
        /* fit aspect aw:ah inside w x h, preserving the smaller dimension */
        long tw = (long)h * aw / ah;
        long th = h;
        if (tw > w) {
            tw = w;
            th = (long)w * ah / aw;
        }
        printf("%ld%c%ld\n", tw, sep, th);
    } else {
        rc = usage(argv[0]);
    }

done:
    SDL_Quit();
    return rc;
}
