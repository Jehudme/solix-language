public class Churn {
    static final class Point {
        public final int coordinate_x;
        public final int coordinate_y;

        public Point(int coordinate_x, int coordinate_y) {
            this.coordinate_x = coordinate_x;
            this.coordinate_y = coordinate_y;
        }
    }

    public static void main(String[] arguments) {
        int total = 0;
        for (int iteration = 0; iteration < 3000000; iteration++) {
            Point point = new Point(iteration, iteration * 2);
            total = (total + point.coordinate_x + point.coordinate_y) & 0x7FFFFFFF;
        }

        if (total == 913288672) {
            System.out.println("RESULT: 913288672");
        } else {
            System.out.println("FAIL");
        }
    }
}
