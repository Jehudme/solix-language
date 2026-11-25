public class StrProc {
    public static int processString(String string_token, int factor_val) {
        if ("alpha".equals(string_token)) {
            return factor_val * 2;
        }
        if ("beta".equals(string_token)) {
            return factor_val * 3;
        }
        if ("gamma".equals(string_token)) {
            return factor_val * 5;
        }
        return factor_val;
    }

    public static void main(String[] arguments) {
        String[] words = new String[] { "alpha", "beta", "gamma" };
        int total = 0;
        for (int iteration = 0; iteration < 100000; iteration++) {
            String word = words[iteration % 3];
            total = (total + processString(word, iteration % 20)) & 0x7FFFFFFF;
        }

        if (total == 3166647) {
            System.out.println("RESULT: 3166647");
        } else {
            System.out.println("FAIL");
        }
    }
}
