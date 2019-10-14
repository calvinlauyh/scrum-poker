export const sleep = (ms: number): Promise<void> => {
    return new Promise((resolve) => {
        setTimeout(resolve, ms);
    });
}

export const nextEventLoop = (): Promise<void> => {
    return new Promise(resolve => {
        setImmediate(resolve);
    });
}
