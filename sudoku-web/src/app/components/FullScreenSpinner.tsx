import { Center, Loader } from "@mantine/core";

export function FullScreenSpinner() {
    return (
        <Center className="app-spinner" h="100%">
            <Loader size="lg" />
        </Center>
    );
}
