public class MatMul {
    public static void main(String[] arguments) {
        int matrix_dim = 60;
        int[] matrix_a = new int[matrix_dim * matrix_dim];
        int[] matrix_b = new int[matrix_dim * matrix_dim];

        for (int row_index = 0; row_index < matrix_dim; row_index++) {
            for (int col_index = 0; col_index < matrix_dim; col_index++) {
                matrix_a[row_index * matrix_dim + col_index] = (row_index + col_index) % 100;
                matrix_b[row_index * matrix_dim + col_index] = (row_index * col_index) % 100;
            }
        }

        int trace = 0;
        for (int row_index = 0; row_index < matrix_dim; row_index++) {
            int cell_value = 0;
            for (int inner_k = 0; inner_k < matrix_dim; inner_k++) {
                cell_value = (cell_value + matrix_a[row_index * matrix_dim + inner_k] * matrix_b[inner_k * matrix_dim + row_index]) & 0x7FFFFFFF;
            }
            trace = (trace + cell_value) & 0x7FFFFFFF;
        }

        if (trace == 9118300) {
            System.out.println("RESULT: 9118300");
        } else {
            System.out.println("FAIL");
        }
    }
}
