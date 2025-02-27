#version 460

#include "shaders/common.glsl"

const uint W = 10;
const uint H = 26;
const uint NUM_BLOCKS = W*H;
const uint FIELD_BYTES = NUM_BLOCKS / 8 + ((NUM_BLOCKS % 8 != 0) ? 1 : 0);
const uint NUM_SHAPES = 7;

const uint LEFT = 1;
const uint RIGHT = 2;
const uint DOWN = 3;

struct TetrominoPositions {
    uint8_t pos[4][2];
};

struct ShapeStartingPositions {
    TetrominoPositions bot_positions[4];
    TetrominoPositions player_position;
};

struct Field {
    uint8_t bytes[FIELD_BYTES];
};

struct MoveResultScore {
    bool game_over;
    uint8_t lines_cleared;
    uint8_t height;
    uint16_t covered;
};

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

layout(set = 0, binding = 0) buffer Sp {SearchParams sp;} search_params;

layout(set = 0, binding = 1) buffer ComputedDropConfigs {
    ComputedDropConfig configs[];
} drop_configs;

layout(set = 0, binding = 2) buffer ShapePositionConfig {
    ShapeStartingPositions starting_positions[NUM_SHAPES];
} spc;

layout(set = 0, binding = 3) buffer Fields {
    Field fields[];
} fields;

layout(set = 0, binding = 4) buffer Scores {
    MoveResultScore scores[];
} scores;

bool try_shift(inout TetrominoPositions tps, uint shift, uint32_t field_idx);
bool is_occupied(uint field_idx, uint8_t x, uint8_t y);
void apply_position(uint field_idx, uint8_t x, uint8_t y);

void main() {
    uint8_t cur_search_depth = search_params.sp.cur_search_depth;

    uint drop_config_idx = config_index(gl_GlobalInvocationID.x, cur_search_depth);
    if (drop_config_idx >= drop_configs.configs.length()) {
      return;
    }

    ComputedDropConfig cfg = drop_configs.configs[drop_config_idx];
    if (cfg.src_field_idx != 0) {
        // Copy the pre-existing score
        scores.scores[drop_config_idx] = scores.scores[cfg.src_field_idx - 1];
    }

    uint8_t shape_idx = search_params.sp.upcoming_shape_idxs[cur_search_depth];

    TetrominoPositions tps = spc.starting_positions[shape_idx].bot_positions[cfg.cw_rotations];

    // Find the amount of shifts after accounting for the walls.
    uint8_t min_x = tps.pos[0][0];
    uint8_t max_x = tps.pos[0][0];
    for (uint i = 0; i < 4; i++) {
        uint8_t x = tps.pos[i][0];
        min_x = min(min_x, x);
        max_x = max(max_x, x);
    }

    // compute the drop
    fields.fields[cfg.dest_field_idx] = fields.fields[cfg.src_field_idx];

    for (uint i = 0; i < cfg.left_shifts; i++) {
        try_shift(tps, LEFT, cfg.dest_field_idx);
    }
    for (uint i = 0; i < cfg.right_shifts; i++) {
        try_shift(tps, RIGHT, cfg.dest_field_idx);
    }
    while (try_shift(tps, DOWN, cfg.dest_field_idx)) {}

    for (uint i = 0; i < 4; i++) {
        apply_position(cfg.dest_field_idx, tps.pos[i][0], tps.pos[i][1]);
    }
}

bool try_shift(inout TetrominoPositions tps, uint shift, uint32_t field_idx) {
    TetrominoPositions next_tps = tps;
    for (uint i = 0; i < 4; i++) {
        if (shift == LEFT) {
            if (next_tps.pos[i][0] == 0) {
                return false;
            }
            next_tps.pos[i][0] -= uint8_t(1);
        } else if (shift == RIGHT) {
            if (next_tps.pos[i][0] == 9) {
                return false;
            }
            next_tps.pos[i][0] += uint8_t(1);
        } else if (shift == DOWN) {
            if (next_tps.pos[i][1] == 0) {
                return false;
            }
            next_tps.pos[i][1] -= uint8_t(1);
        }

        if (is_occupied(field_idx, next_tps.pos[i][0], next_tps.pos[i][1])) {
            return false;
        }
    }
    tps = next_tps;
    return true;
}

bool is_occupied(uint field_idx, uint8_t x, uint8_t y) {
    uint bit_index = y * W + x;
    uint byte_index = bit_index / 8;
    uint offset = bit_index % 8;
    uint8_t mask = uint8_t(1) << offset;

    return (fields.fields[field_idx].bytes[byte_index] & mask) != 0;
}

void apply_position(uint field_idx, uint8_t x, uint8_t y) {
    uint bit_index = y * W + x;
    uint byte_index = bit_index / 8;
    uint offset = bit_index % 8;
    uint8_t mask = uint8_t(1) << offset;

    fields.fields[field_idx].bytes[byte_index] |= mask;
}
