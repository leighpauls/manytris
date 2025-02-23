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

}
