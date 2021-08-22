import React from 'react';
import useMediaQuery from '@material-ui/core/useMediaQuery';
import {createTheme, ThemeProvider} from '@material-ui/core/styles';
import CssBaseline from '@material-ui/core/CssBaseline';

export const Theme: React.FunctionComponent = ({children}) => {
  const prefersDarkMode = useMediaQuery('(prefers-color-scheme: dark)');

  const theme = React.useMemo(
    () =>
      createTheme({
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
      }),
    [prefersDarkMode],
  );

  return (
    <ThemeProvider theme={theme}>
      <CssBaseline/>
      {children}
    </ThemeProvider>
  );
}