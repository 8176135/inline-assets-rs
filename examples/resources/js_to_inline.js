function valOrPlaceholder(item) {
    if (item.value) {
        return item.value;
    } else {
        return item.placeholder;
    }
}