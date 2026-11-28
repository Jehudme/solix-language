public class Fib {
    public static int fibonacci(int number) {
        if (number <= 1) {
            return number;
        }
        return fibonacci(number - 1) + fibonacci(number - 2);
    }

    public static void main(String[] arguments) {
        int result = fibonacci(32);
        if (result == 2178309) {
            System.out.println("RESULT: 2178309");
        } else {
            System.out.println("FAIL");
        }
    }
}
