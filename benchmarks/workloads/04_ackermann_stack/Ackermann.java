public class Ackermann {
    public static int ackermann(int parameter_m, int parameter_n) {
        if (parameter_m == 0) {
            return parameter_n + 1;
        }
        if (parameter_n == 0) {
            return ackermann(parameter_m - 1, 1);
        }
        return ackermann(parameter_m - 1, ackermann(parameter_m, parameter_n - 1));
    }

    public static void main(String[] arguments) {
        int result = ackermann(3, 6);
        if (result == 509) {
            System.out.println("RESULT: 509");
        } else {
            System.out.println("FAIL");
        }
    }
}
