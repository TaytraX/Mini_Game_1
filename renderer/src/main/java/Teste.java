public class Teste {
    public static native void updateValue(double x, double y, double z);
    public static native void render();
    public static native void updateChunk(float[] chunkData);

    static {
        System.loadLibrary("rendering");
    }
}