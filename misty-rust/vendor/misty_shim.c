// Petite couche d'accès à cubiomes (MIT, voir cubiomes/LICENSE) pour le visualiseur de seed de Misty.
#include "cubiomes/generator.h"
#include "cubiomes/finders.h"
#include "cubiomes/util.h"
#include <stdint.h>
#include <stdlib.h>
#include <string.h>

// Biomes d'une zone : sx*sz cases de `scale` blocs, la case (0,0) étant en (x, z) (en cases).
// Renvoie 0 si tout va bien.
int misty_biomes(int mc, int dim, int64_t seed, int scale, int x, int z, int sx, int sz, int *sortie)
{
    Generator g;
    setupGenerator(&g, mc, 0);
    applySeed(&g, dim, (uint64_t)seed);
    Range r;
    r.scale = scale;
    r.x = x; r.z = z; r.sx = sx; r.sz = sz;
    r.y = dim == 0 ? (scale == 1 ? 63 : 63 / 4) : 0;
    r.sy = 1;
    // genBiomes a besoin d'un tampon de travail plus grand que sx*sz : on passe par allocCache
    int *cache = allocCache(&g, r);
    if (!cache) return 1;
    int erreur = genBiomes(&g, cache, r);
    if (!erreur) memcpy(sortie, cache, (size_t)sx * (size_t)sz * sizeof(int));
    free(cache);
    return erreur;
}

void misty_couleurs(unsigned char *sortie768)
{
    unsigned char c[256][3];
    initBiomeColors(c);
    for (int i = 0; i < 256; i++) { sortie768[i*3] = c[i][0]; sortie768[i*3+1] = c[i][1]; sortie768[i*3+2] = c[i][2]; }
}

const char *misty_nom_biome(int mc, int id) { return biome2str(mc, id); }

void misty_spawn(int mc, int64_t seed, int *x, int *z)
{
    Generator g;
    setupGenerator(&g, mc, 0);
    applySeed(&g, 0, (uint64_t)seed);
    Pos p = estimateSpawn(&g, NULL);
    *x = p.x; *z = p.z;
}

// Structures d'un type dans le rectangle de blocs [x0,x1] x [z0,z1] ; renvoie leur nombre (au plus `max`).
int misty_structures(int mc, int dim, int64_t seed, int type, int x0, int z0, int x1, int z1, int *sortie, int max)
{
    StructureConfig sc;
    if (!getStructureConfig(type, mc, &sc)) return 0;
    if (sc.dim != dim) return 0;
    Generator g;
    setupGenerator(&g, mc, 0);
    applySeed(&g, dim, (uint64_t)seed);
    int taille = sc.regionSize * 16;
    int rx0 = (int)((x0 >= 0 ? x0 : x0 - taille + 1) / taille) - 1, rx1 = (int)((x1 >= 0 ? x1 : x1 - taille + 1) / taille) + 1;
    int rz0 = (int)((z0 >= 0 ? z0 : z0 - taille + 1) / taille) - 1, rz1 = (int)((z1 >= 0 ? z1 : z1 - taille + 1) / taille) + 1;
    int n = 0;
    for (int rz = rz0; rz <= rz1; rz++)
        for (int rx = rx0; rx <= rx1; rx++) {
            Pos p;
            if (!getStructurePos(type, mc, (uint64_t)seed, rx, rz, &p)) continue;
            if (p.x < x0 || p.x > x1 || p.z < z0 || p.z > z1) continue;
            if (!isViableStructurePos(type, &g, p.x, p.z, 0)) continue;
            if (n < max) { sortie[n*2] = p.x; sortie[n*2+1] = p.z; n++; }
        }
    return n;
}
