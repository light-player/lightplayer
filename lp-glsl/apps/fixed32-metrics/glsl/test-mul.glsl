vec4 main(vec2 fragCoord, vec2 outputSize, float time) {
    float a = 1.5;
    float b = 2.3;
    float c = a * b;
    float d = c * 0.7;
    float e = d * a;
    
    return vec4(c, d, e, 1.0);
}
