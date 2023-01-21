#version 330 core

in vec2 v_uv;

out vec4 color;

uniform vec2 u_dimensions;
uniform vec4 u_color = vec4(1, 0.39, 0.46, 1);
uniform float u_radius = 10;
uniform float u_zoom;

float roundRect(vec2 coords, float radius, vec2 dimensions, float zoom) {
  float mask = 1.0;

  if ((coords.x < radius || coords.x > dimensions.x - radius) &&
      (coords.y < radius || coords.y > dimensions.y - radius)) {
    vec2 corner = radius - min(coords, dimensions - coords);
    float dist = length(corner);
    mask =  1.0 - smoothstep(radius - 0.75 / zoom, radius + 0.75 / zoom, dist);
  }

  return mask;
}

void main() {
  float rrMask = roundRect(v_uv * u_dimensions, u_radius, u_dimensions, u_zoom);
  color = vec4(u_color.rgb, u_color.a * rrMask);
}