public class Closures {
    interface IntUnaryOperator {
        int apply(int input_val);
    }

    public static void main(String[] arguments) {
        int base_value = 7;
        IntUnaryOperator adder = (int input_val) -> input_val + base_value;

        int total = 0;
        for (int iteration = 0; iteration < 50000; iteration++) {
            total = (total + adder.apply(iteration % 100)) & 0x7FFFFFFF;
        }

        if (total == 2825000) {
            System.out.println("RESULT: 2825000");
        } else {
            System.out.println("FAIL");
        }
    }
}
