public class Sieve {
    public static void main(String[] arguments) {
        int limit = 100000;
        boolean[] is_prime = new boolean[limit + 1];
        for (int number = 2; number <= limit; number++) {
            is_prime[number] = true;
        }

        for (int prime_factor = 2; prime_factor * prime_factor <= limit; prime_factor++) {
            if (is_prime[prime_factor]) {
                for (int multiple = prime_factor * prime_factor; multiple <= limit; multiple += prime_factor) {
                    is_prime[multiple] = false;
                }
            }
        }

        int count = 0;
        for (int number = 2; number <= limit; number++) {
            if (is_prime[number]) {
                count++;
            }
        }

        if (count == 9592) {
            System.out.println("RESULT: 9592");
        } else {
            System.out.println("FAIL");
        }
    }
}
