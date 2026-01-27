import { describe, it, expect } from 'vitest';
import { render } from '@testing-library/react';
import Button from '../../components/Button';
import Card from '../../components/Card';
import StatusMessage from '../../components/StatusMessage';
import React from 'react';

describe('Component Coverage', () => {
  describe('Button', () => {
    it('should render button', () => {
      const { getByText } = render(
        React.createElement(Button, { onClick: () => {} }, 'Click me')
      );
      expect(getByText('Click me')).toBeDefined();
    });
  });

  describe('Card', () => {
    it('should render card', () => {
      const { container } = render(React.createElement(Card));
      expect(container.querySelector('.bg-white')).toBeDefined();
    });
  });

  describe('StatusMessage', () => {
    it('should render success message', () => {
      const { getByText } = render(
        React.createElement(StatusMessage, {
          type: 'success',
          message: 'Success!',
        })
      );
      expect(getByText('Success!')).toBeDefined();
    });

    it('should render error message', () => {
      const { getByText } = render(
        React.createElement(StatusMessage, {
          type: 'error',
          message: 'Error occurred',
        })
      );
      expect(getByText('Error occurred')).toBeDefined();
    });

    it('should render loading message', () => {
      const { getByText } = render(
        React.createElement(StatusMessage, {
          type: 'loading',
          message: 'Loading...',
        })
      );
      expect(getByText('Loading...')).toBeDefined();
    });
  });
});
