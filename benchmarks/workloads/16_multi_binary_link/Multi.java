public class Multi {
    public static int compute(int coordinate_x, int coordinate_y) {
        return coordinate_x * 3 + coordinate_y * 7;
    }

    public static void main(String[] arguments) {
        int total = 0;
        for (int iteration = 0; iteration < 3000000; iteration++) {
            total = (total + compute(iteration % 100, iteration % 50)) & 0x7FFFFFFF;
        }

        if (total == 960000000) {
            System.out.println("RESULT: 960000000");
        } else {
            System.out.println("FAIL");
        }
    }
}
