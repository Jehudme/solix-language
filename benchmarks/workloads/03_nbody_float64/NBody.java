public class NBody {
    public static void main(String[] arguments) {
        double position_x = 0.5;
        double position_y = 1.2;
        double velocity_x = 0.01;
        double velocity_y = 0.02;

        for (int step = 0; step < 2000000; step++) {
            double distance_squared = position_x * position_x + position_y * position_y + 0.1;
            double force = 1.0 / distance_squared;
            velocity_x += force * 0.001;
            velocity_y += force * 0.001;
            position_x += velocity_x;
            position_y += velocity_y;
        }

        int energy = (int) (position_x * 100.0) + (int) (position_y * 100.0);
        if (energy == 14340304) {
            System.out.println("RESULT: 14340304");
        } else {
            System.out.println("FAIL");
        }
    }
}
