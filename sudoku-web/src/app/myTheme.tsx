import React from 'react';
import useMediaQuery from '@mui/material/useMediaQuery';
import {adaptV4Theme, createTheme, StyledEngineProvider, Theme, ThemeProvider} from '@mui/material/styles';
import CssBaseline from '@mui/material/CssBaseline';


declare module '@mui/styles/defaultTheme' {
  // eslint-disable-next-line @typescript-eslint/no-empty-interface
  interface DefaultTheme extends Theme {
  }
}


export const MyTheme: React.FunctionComponent = ({children}) => {
  const prefersDarkMode = useMediaQuery('(prefers-color-scheme: dark)');

  const theme = React.useMemo(
    () =>
      createTheme(adaptV4Theme({
        palette: {
          primary: {
            main: prefersDarkMode ? "#5FA1F2FF" : "#0D4FA0"
          },
          type: prefersDarkMode ? 'dark' : 'light',
          background: prefersDarkMode ? {
            default: "#121212",
            paper: "#2C2C2C"
          } : {}
        },
      })),
    [prefersDarkMode],
  );

  return (
    <StyledEngineProvider injectFirst>
      <ThemeProvider theme={theme}>
        <CssBaseline/>
        {children}
      </ThemeProvider>
    </StyledEngineProvider>
  );
}