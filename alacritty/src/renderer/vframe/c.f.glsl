#version 330 core
uniform vec2 iResolution;
uniform sampler2D iChannel0;

out vec4 fragColor;

void mainImage(out vec4 fragColor, in vec2 fragCoord)
{
    fragColor = texture(iChannel0, fragCoord.xy / iResolution.xy);
}

void main() {
    mainImage(fragColor, gl_FragCoord.xy);
}
