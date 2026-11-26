public class Fib {
    public static int fibonacci(int number) {
        if (number <= 1) {
            return number;
        }
        return fibonacci(number - 1) + fibonacci(number - 2);
    }

    public static void main(String[] arguments) {
        int result = fibonacci(28);
        if (result == 317811) {
            System.out.println("RESULT: 317811");
        } else {
            System.out.println("FAIL");
        }
    }
}
