import org.joml.Quaterniond;
import org.joml.Vector3d;

import java.util.HashMap;

public class ComponentScheduler {
    public HashMap<Integer, Vector3d> positionComponent = new HashMap<>();
    public HashMap<Integer, Quaterniond> rotateComponent = new HashMap<>();
    public HashMap<Integer, Double> healtComponent = new HashMap<>();
    public HashMap<Integer, Attribute> attributeComponent = new HashMap<>();
}