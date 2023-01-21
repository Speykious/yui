#version 330 core

layout(location = 0) in vec4 position;
layout(location = 1) in vec2 uv;

out vec2 v_uv;

uniform mat4 u_mvp;

void main() {
  gl_Position = u_mvp * position;
  v_uv = uv;
}