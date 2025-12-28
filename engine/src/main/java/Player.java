import org.joml.Quaterniond;
import org.joml.Vector3d;

public class Player extends Entity {
    public final int ID;
    Vector3d position;
    Quaterniond rotation;

    public Player(int ID, ComponentScheduler component) {
        this.ID = ID;
        position = component.positionComponent.get(ID);
        rotation = component.rotateComponent.get(ID);
    }
}