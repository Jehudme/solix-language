public class BTrees {
    static final class TreeNode {
        public final int item;
        public TreeNode left;
        public TreeNode right;

        public TreeNode(int item) {
            this.item = item;
            this.left = null;
            this.right = null;
        }
    }

    public static TreeNode buildTree(int depth, int item) {
        TreeNode node = new TreeNode(item);
        if (depth > 0) {
            node.left = buildTree(depth - 1, item * 2);
            node.right = buildTree(depth - 1, item * 2 + 1);
        }
        return node;
    }

    public static int checkTree(TreeNode node) {
        if (node == null) {
            return 0;
        }
        return (node.item + checkTree(node.left) + checkTree(node.right)) & 0x7FFFFFFF;
    }

    public static void main(String[] arguments) {
        int total = 0;
        for (int iteration = 0; iteration < 10; iteration++) {
            TreeNode root = buildTree(10, iteration + 1);
            total = (total + checkTree(root)) & 0x7FFFFFFF;
        }

        if (total == 687761806) {
            System.out.println("RESULT: 687761806");
        } else {
            System.out.println("FAIL");
        }
    }
}
