#version 460

#include "shaders/common.glsl"

const uint W = 10;
const uint H = 26;
const uint NUM_BLOCKS = W*H;
const uint FIELD_BYTES = NUM_BLOCKS / 8 + ((NUM_BLOCKS % 8 != 0) ? 1 : 0);
const uint NUM_SHAPES = 7;

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

    uint8_t left_shifts = min(cfg.left_shifts, min_x);
    uint8_t right_shifts = min(cfg.right_shifts, uint8_t(9) - max_x);

    for (uint i = 0; i < 4; i++) {
        tps.pos[i][0] -= left_shifts;
        tps.pos[i][0] += right_shifts;
    }

    // compute the drop
    fields.fields[cfg.dest_field_idx] = fields.fields[cfg.src_field_idx];
    // TODO
}
