#include <stdio.h>
#include <stdlib.h>

typedef struct Node {
    int item;
    struct Node *left;
    struct Node *right;
} Node;

static Node *build_tree(int depth, int item) {
    Node *node = (Node *)malloc(sizeof(Node));
    node->item = item;
    node->left = NULL;
    node->right = NULL;
    if (depth > 0) {
        node->left = build_tree(depth - 1, item * 2);
        node->right = build_tree(depth - 1, item * 2 + 1);
    }
    return node;
}

static int check_tree(Node *node) {
    if (!node) {
        return 0;
    }
    return (node->item + check_tree(node->left) + check_tree(node->right)) & 0x7FFFFFFF;
}

static void free_tree(Node *node) {
    if (!node) {
        return;
    }
    free_tree(node->left);
    free_tree(node->right);
    free(node);
}

int main(void) {
    int total = 0;
    for (int i = 0; i < 100; i++) {
        Node *root = build_tree(10, i + 1);
        total = (total + check_tree(root)) & 0x7FFFFFFF;
        free_tree(root);
    }

    if (total == 687761806) {
        printf("RESULT: 687761806\n");
    } else {
        printf("FAIL\n");
    }
    return 0;
}
