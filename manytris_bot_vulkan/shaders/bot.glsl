#version 460
#extension GL_EXT_shader_8bit_storage : enable
#extension GL_EXT_shader_explicit_arithmetic_types : enable

layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;

const int MAX_SEARCH_DEPTH = 6;

layout(set = 0, binding = 0) buffer SearchParams {
    uint8_t cur_search_depth;
    uint8_t upcoming_shape_idxs[MAX_SEARCH_DEPTH + 1];
} search_params;

struct DropConfig {
    uint32_t tetromino_idx;
    uint32_t next_tetromino_idx;
    uint32_t initial_field_idx;
    uint32_t dest_field_idx;
    uint8_t left_shifts;
    uint8_t right_shifts;
};

layout(set = 0, binding = 1) buffer DropConfigs {
    DropConfig configs[];
} drop_configs;


void main() {
    uint idx = gl_GlobalInvocationID.x;
    if (idx >= drop_configs.configs.length()) {
      return;
    }
    drop_configs.configs[idx] = DropConfig(
      idx, idx, idx, idx, search_params.cur_search_depth, uint8_t(2)
    );
}
